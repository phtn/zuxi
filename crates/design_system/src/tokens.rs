//! Raw design tokens. Values in this module are deliberately GPUI-agnostic.

pub mod color {
    pub const WHITE: u32 = 0xffffff;
    // sRGB equivalent of `oklch(0.26 0.004 264.5)`.
    pub const DARK_BACKGROUND: u32 = 0x232426;
    pub const NEUTRAL_50: u32 = 0xf5f5f7;
    pub const NEUTRAL_100: u32 = 0xe5e5ea;
    pub const NEUTRAL_200: u32 = 0xd1d1d6;
    pub const NEUTRAL_300: u32 = 0xc7c7cc;
    pub const NEUTRAL_400: u32 = 0xaeaeb2;
    pub const NEUTRAL_500: u32 = 0x8e8e93;
    pub const NEUTRAL_600: u32 = 0x636366;
    pub const NEUTRAL_700: u32 = 0x48484a;
    pub const NEUTRAL_800: u32 = 0x3a3a3c;
    pub const NEUTRAL_900: u32 = 0x2c2c2e;
    pub const NEUTRAL_950: u32 = 0x1c1c1e;

    pub const BLUE_400: u32 = 0x409cff;
    pub const BLUE_500: u32 = 0x0a84ff;
    pub const BLUE_600: u32 = 0x007aff;
    pub const BLUE_700: u32 = 0x0066cc;

    pub const RED_400: u32 = 0xff6961;
    pub const RED_500: u32 = 0xff453a;
    pub const RED_600: u32 = 0xff3b30;
    pub const RED_700: u32 = 0xd70015;

    pub const SNOW_A: u32 = 0x001D51;
    pub const SNOW_B: u32 = 0xFFE3A5;

    pub const ARC_A: u32 = 0xF9D74A;
    pub const ARC_B: u32 = 0x1E1E1C;

    pub const D350: u32 = 0xC3D8C5;
    pub const D351: u32 = 0xFAFAFA;
    pub const D352: u32 = 0x51504F;
    pub const D353: u32 = 0xA8C0B2;
    pub const D354: u32 = 0xFF7247;
}

pub mod spacing {
    pub const XXS: f32 = 4.0;
    pub const XS: f32 = 8.0;
    pub const SM: f32 = 12.0;
    pub const MD: f32 = 16.0;
    pub const LG: f32 = 24.0;
    pub const XL: f32 = 32.0;
    pub const XXL: f32 = 48.0;
}

pub mod typography {
    pub const SIZE_XS: f32 = 10.0;
    pub const SIZE_SM: f32 = 13.0;
    pub const SIZE_BODY: f32 = 15.0;
    pub const SIZE_HEADING: f32 = 24.0;

    pub const LINE_HEIGHT_SM: f32 = 18.0;
    pub const LINE_HEIGHT_BODY: f32 = 22.0;
    pub const LINE_HEIGHT_HEADING: f32 = 30.0;

    pub const WEIGHT_THIN: f32 = 300.0;
    pub const WEIGHT_LIGHT: f32 = 300.0;
    pub const WEIGHT_REGULAR: f32 = 400.0;
    pub const WEIGHT_MEDIUM: f32 = 500.0;
    pub const WEIGHT_SEMIBOLD: f32 = 600.0;
}

pub mod radius {
    pub const SM: f32 = 4.0;
    pub const MD: f32 = 8.0;
    pub const LG: f32 = 12.0;
}

pub mod opacity {
    pub const DISABLED: f32 = 0.5;
    pub const HOVER: f32 = 0.08;
    pub const SELECTED: f32 = 0.14;
    pub const SHADOW: f32 = 0.16;
}

const _: () = {
    assert!(spacing::XXS < spacing::XS);
    assert!(spacing::XS < spacing::SM);
    assert!(spacing::SM < spacing::MD);
    assert!(spacing::MD < spacing::LG);
    assert!(spacing::LG < spacing::XL);
    assert!(spacing::XL < spacing::XXL);

    assert!(typography::SIZE_SM < typography::SIZE_BODY);
    assert!(typography::SIZE_BODY < typography::SIZE_HEADING);
};
