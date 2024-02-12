use crate::file::crpf_struct;
use crate::file::enums::PixelFormat;

crpf_struct! {
    ContentHash {
        hash0("hash0"): UInt32,
        hash1("hash1"): UInt32,
        hash2("hash2"): UInt32,
        size("size"): UInt32,
    }
}

crpf_struct! {
    SoundResource {
        channel_configuration("channelConfiguration"): Enum,
        frames_per_second("framesPerSecond"): UInt16,
        frame_count("frameCount"): UInt32,
        duration("duration"): Time,
        data_hash("dataHash"): ContentHash,
        debug_name("debugName"): BlobString,
    }
}

crpf_struct! {
    Time {
        value("value"): SInt64,
    }
}

crpf_struct! {
    RenderMaterialResource {
        effect_id("effectId"): Enum,
        flags("flags"): Bitmask8,
        data("data"): BlobArray<UInt8>,
        data_type_signature("dataTypeSignature"): HashKey32,
        images("images"): BlobArray<RenderMaterialImage>,
        debug_name("debugName"): BlobString,
    }
}

crpf_struct! {
    HashKey32 {
        value("value"): UInt32,
    }
}

crpf_struct! {
    RenderMaterialImage {
        data_offset("dataOffset"): UInt32,
        width("width"): UInt16,
        height("height"): UInt16,
        depth("depth"): UInt16,
        array_size("arraySize"): UInt16,
        level_count("levelCount"): UInt8,
        texture_type("type"): Enum,
        format("format"): PixelFormat,
        data("data"): ContentHash,
        enable_streaming("enableStreaming"): Bool,
        debug_name("debugName"): BlobString,
    }
}

crpf_struct! {
    ItemIconRegistryResource {
        icons("icons"): BlobArray<ItemIconRegistryEntryResource>,
    }
}

crpf_struct! {
    ItemIconRegistryEntryResource {
        guid("guid"): Guid,
        ui_texture("uiTexture"): UiTextureResource,
    }
}

crpf_struct! {
    UiTextureResource {
        level_count("levelCount"): UInt8,
        format("format"): PixelFormat,
        size("size"): UInt2,
        data("data"): ContentHash,
    }
}

crpf_struct! {
    UInt2 {
        x("x"): UInt32,
        y("y"): UInt32,
    }
}
