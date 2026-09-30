use core::mem;

use ku::{
    log::debug,
    memory::{
        Block,
        block::Memory,
    },
};

use crate::{
    BARS_END_ADDRESS,
    BARS_START_ADDRESS,
    Bar,
    ConfigSpace,
    Device,
    Kind,
    RoutingId,
    bar::{
        COMMAND_ADDRESS,
        Width,
    },
};

pub(super) struct MockDevice {
    config_space: MockConfigSpace,

    vendor: &'static str,
    device: &'static str,
    revision: u8,

    subvendor_subdevice: Option<(&'static str, u16)>,

    class: &'static str,
    subclass: &'static str,
    interface: u8,

    is_multi_function: bool,

    kind: Kind,
}

impl MockDevice {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        config_space: [u8; MockConfigSpace::COUNT],
        bars: [Option<Bar>; 6],
        vendor: &'static str,
        device: &'static str,
        revision: u8,
        subvendor_subdevice: Option<(&'static str, u16)>,
        class: &'static str,
        subclass: &'static str,
        interface: u8,
        is_multi_function: bool,
        kind: Kind,
    ) -> Self {
        Self {
            config_space: MockConfigSpace::new(config_space, bars),
            vendor,
            device,
            revision,
            subvendor_subdevice,
            class,
            subclass,
            interface,
            is_multi_function,
            kind,
        }
    }

    pub(super) fn name(&self) -> &str {
        self.device
    }

    pub(super) fn device(&mut self) -> Device {
        Device::new(&mut self.config_space, RoutingId::new(0, 0, 0)).unwrap()
    }

    pub(super) fn validate_device(&mut self) {
        let id = *self.device().id();

        assert_eq!(id.vendor().name().unwrap(), self.vendor);
        assert_eq!(id.device().name().unwrap(), self.device);
        assert_eq!(*id.revision(), self.revision);
    }

    pub(super) fn validate_subdevice(&mut self) {
        let device = self.device();

        if let Some((subvendor, subdevice)) = self.subvendor_subdevice {
            assert_eq!(device.subvendor().unwrap().name().unwrap(), subvendor);
            assert_eq!(device.subdevice().unwrap().id(), subdevice);
        }
    }

    pub(super) fn validate_class(&mut self) {
        let class = *self.device().class();

        assert_eq!(class.class().name().unwrap(), self.class);
        assert_eq!(class.subclass().name().unwrap(), self.subclass);
        assert_eq!(class.interface().id(), self.interface);
    }

    pub(super) fn validate_bars(&mut self) {
        let device = self.device();

        match device.kind() {
            &Kind::Normal { bars } => validate_bars(bars, self.config_space.bars()),
            _ => panic!("unexpected device type {}", device.kind()),
        }
    }

    pub(super) fn validate(&mut self) {
        self.validate_device();
        self.validate_subdevice();
        self.validate_class();

        let device = self.device();

        assert_eq!(*device.is_multi_function(), self.is_multi_function);
        assert_eq!(
            mem::discriminant(device.kind()),
            mem::discriminant(&self.kind),
        );

        match *device.kind() {
            Kind::Normal { bars } => validate_bars(bars, self.config_space.bars()),
            Kind::PciPciBridge {
                bars,
                primary,
                secondary,
                subordinate,
                io_behind_bridge,
                memory_behind_bridge,
                prefetchable_memory_behind_bridge,
            } => {
                let Kind::PciPciBridge {
                    bars: expected_bars,
                    primary: expected_primary,
                    secondary: expected_secondary,
                    subordinate: expected_subordinate,
                    io_behind_bridge: expected_io_behind_bridge,
                    memory_behind_bridge: expected_memory_behind_bridge,
                    prefetchable_memory_behind_bridge: expected_prefetchable_memory_behind_bridge,
                } = self.kind
                else {
                    panic!("expected kind: {}", self.kind);
                };

                assert_eq!(primary, expected_primary);
                assert_eq!(secondary, expected_secondary);
                assert_eq!(subordinate, expected_subordinate);
                assert_eq!(io_behind_bridge, expected_io_behind_bridge);
                assert_eq!(memory_behind_bridge, expected_memory_behind_bridge);
                assert_eq!(
                    prefetchable_memory_behind_bridge,
                    expected_prefetchable_memory_behind_bridge,
                );

                validate_bars(bars, expected_bars);
            },
            _ => panic!("unexpected device type {}", device.kind()),
        }
    }
}

impl Bar {
    fn info(bar: &MockBar) -> u32 {
        match *bar {
            MockBar::High(_) => 0x0,

            MockBar::Low(Bar::Memory {
                prefetchable,
                width,
                ..
            }) => {
                let prefetchable_mask = if prefetchable {
                    0x8
                } else {
                    0x0
                };
                let address_width_mask = if width == Width::Memory64 {
                    0x4
                } else {
                    0x0
                };

                prefetchable_mask | address_width_mask
            },

            MockBar::Low(Bar::Port { .. }) => 0x1,

            MockBar::None => 0x0,
        }
    }

    fn mask(bar: &MockBar) -> u32 {
        return match *bar {
            MockBar::High(Bar::Memory { block, .. }) => (block_mask(block) >> u32::BITS) as u32,
            MockBar::High(Bar::Port { .. }) => panic!("x86-64 has 16-bit I/O space"),
            MockBar::Low(Bar::Memory { block, .. }) => 0xF | block_mask(block) as u32,
            MockBar::Low(Bar::Port { block, .. }) => 0x1 | block_mask(block) as u32,
            MockBar::None => !0x0,
        };

        fn block_mask<T: Memory>(block: Block<T>) -> usize {
            if block.is_empty() {
                panic!("empty blocks are not allowed in BARs")
            } else {
                block.size() - 1
            }
        }
    }
}

struct MockConfigSpace {
    bars: [MockBar; 6],
    data: [u8; Self::COUNT],
}

impl MockConfigSpace {
    fn new(
        data: [u8; Self::COUNT],
        bars: [Option<Bar>; 6],
    ) -> Self {
        let mut mock_bars = [MockBar::None; 6];
        let mut i = 0;
        while i < bars.len() {
            if let Some(bar) = bars[i] {
                mock_bars[i] = MockBar::Low(bar);
                if let Bar::Memory { width, .. } = bar &&
                    width == Width::Memory64
                {
                    i += 1;
                    assert_eq!(bars[i], None);
                    mock_bars[i] = MockBar::High(bar);
                }
            }
            i += 1;
        }

        Self {
            bars: mock_bars,
            data,
        }
    }

    fn bars(&self) -> [Option<Bar>; 6] {
        let mut bars = [None; 6];
        for (bar, mock_bar) in bars.iter_mut().zip(self.bars) {
            if let MockBar::Low(mock_bar) = mock_bar {
                *bar = Some(mock_bar);
            }
        }

        bars
    }

    const COUNT: usize = 256;
}

impl ConfigSpace for MockConfigSpace {
    unsafe fn read(
        &mut self,
        _routing_id: RoutingId,
        offset: usize,
    ) -> u32 {
        u32::from_le_bytes(
            self.data[offset .. offset + mem::size_of::<u32>()]
                .try_into()
                .expect("slice len mismatch"),
        )
    }

    unsafe fn write(
        &mut self,
        _routing_id: RoutingId,
        offset: usize,
        mut data: u32,
    ) {
        if (BARS_START_ADDRESS .. BARS_END_ADDRESS).contains(&offset) {
            assert_eq!(
                self.data[COMMAND_ADDRESS] & 0x3,
                0,
                "disable I/O and memory decode before writing to BARs",
            );

            let bar = &self.bars[(offset - BARS_START_ADDRESS) / mem::size_of::<u32>()];
            data = (Bar::info(bar) & Bar::mask(bar)) | (data & !Bar::mask(bar));
        }

        self.data[offset .. offset + mem::size_of::<u32>()].copy_from_slice(&data.to_le_bytes());
    }
}

fn validate_bars<const N: usize>(
    bars: [Option<Bar>; N],
    expected_bars: [Option<Bar>; N],
) {
    let mut i = 0;
    let mut j = 0;

    while i < bars.len() && j < expected_bars.len() {
        if bars[i].is_none() {
            i += 1;
        } else if expected_bars[j].is_none() {
            j += 1;
        } else {
            debug!(expected_bar = ?expected_bars[j], "    ");
            debug!(bar = ?bars[i], "             ");

            assert_eq!(bars[i], expected_bars[j]);
            i += 1;
            j += 1;
        }
    }

    assert!(bars[i ..].iter().all(Option::is_none));
    assert!(expected_bars[j ..].iter().all(Option::is_none));
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MockBar {
    High(Bar),
    Low(Bar),
    None,
}
