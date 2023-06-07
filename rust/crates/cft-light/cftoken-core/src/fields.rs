use plugin_transactor::SField;
use rippled_bridge::type_ids::SerializedTypeID;

pub trait CFTokenFields {
    fn sf_asset_code() -> Self;
    fn sf_maximum_amount() -> Self;
    fn sf_outstanding_amount() -> Self;
    fn sf_locked_amount() -> Self;
    fn sf_cft_metadata() -> Self;
    fn sf_asset_scale() -> Self;
    fn sf_issuance_id() -> Self;
    fn sf_cft_amount() -> Self;
    fn sf_cf_tokens() -> Self;
    fn sf_cf_token() -> Self;
}

impl CFTokenFields for SField<'_> {
    fn sf_asset_code() -> Self {
        SField::get_plugin_field(SerializedTypeID::STI_UINT160, 5)
    }

    fn sf_maximum_amount() -> Self {
        SField::get_plugin_field(SerializedTypeID::STI_UINT64, 20)
    }

    fn sf_outstanding_amount() -> Self {
        SField::get_plugin_field(SerializedTypeID::STI_UINT64, 21)
    }

    fn sf_locked_amount() -> Self {
        SField::get_plugin_field(SerializedTypeID::STI_UINT64, 22)
    }

    fn sf_cft_metadata() -> Self {
        SField::get_plugin_field(SerializedTypeID::STI_VL, 22)
    }

    fn sf_asset_scale() -> Self {
        SField::get_plugin_field(SerializedTypeID::STI_UINT8, 19)
    }

    fn sf_issuance_id() -> Self {
        SField::get_plugin_field(SerializedTypeID::STI_UINT256, 28)
    }

    fn sf_cft_amount() -> Self {
        SField::get_plugin_field(SerializedTypeID::STI_UINT64, 23)
    }

    fn sf_cf_tokens() -> Self {
        SField::get_plugin_field(SerializedTypeID::STI_ARRAY, 21)
    }

    fn sf_cf_token() -> Self {
        SField::get_plugin_field(SerializedTypeID::STI_OBJECT, 25)
    }
}