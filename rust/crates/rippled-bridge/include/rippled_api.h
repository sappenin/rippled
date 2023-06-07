//
// Created by Noah Kramer on 4/17/23.
//

#ifndef PLUGIN_TRANSACTOR_BLOBSTORE_H
#define PLUGIN_TRANSACTOR_BLOBSTORE_H

#pragma once

#include <memory>
#include "ripple/basics/base64.h"
#include <ripple/app/tx/impl/Transactor.h>
#include <ripple/protocol/st.h>
#include <ripple/protocol/TxFlags.h>
#include <ripple/protocol/Feature.h>
#include <ripple/ledger/View.h>
#include <ripple/protocol/InnerObjectFormats.h>
#include "rust/cxx.h"

std::unique_ptr <std::string>
base64_decode_ptr(std::string const &data);

std::unique_ptr <ripple::NotTEC>
from_tefcodes(ripple::TEFcodes code);

std::unique_ptr <ripple::STTx> tx_ptr(ripple::PreflightContext const &ctx);

// Return the XRP Issuer as an AccountID
ripple::AccountID const &xrp_account();

ripple::STTx const &get_dummy_sttx();

ripple::uint256 const &fixMasterKeyAsRegularKey();

ripple::PreflightContext const &get_dummy_ctx();

ripple::XRPAmount defaultCalculateBaseFee(ripple::ReadView const& view, ripple::STTx const& tx);

ripple::XRPAmount minimumFee(
        ripple::Application& app,
        ripple::XRPAmount baseFee,
        ripple::Fees const& fees,
        ripple::ApplyFlags flags
        );

bool setFlag(
        std::shared_ptr<ripple::SLE>const & sle,
        std::uint32_t f);

void setAccountID(
        std::shared_ptr<ripple::SLE>const & sle,
        ripple::SField const& field,
                ripple::AccountID const& v
);

void setFieldAmountXRP(
    std::shared_ptr<ripple::SLE>const & sle,
    ripple::SField const& field,
    ripple::XRPAmount const& xrpAmount
);

void setPluginType(
        std::shared_ptr<ripple::SLE>const & sle,
        ripple::SField const& field,
        ripple::STPluginType const& v
);

void setFieldArray(
    std::shared_ptr<ripple::SLE>const& sle,
    ripple::SField const& field,
    std::unique_ptr<ripple::STArray> value
    );

void setFieldU8(
    std::shared_ptr<ripple::SLE>const & sle,
    ripple::SField const& field,
    std::uint8_t v
);
void setFieldU16(
    std::shared_ptr<ripple::SLE>const & sle,
    ripple::SField const& field,
    std::uint16_t v
);
void setFieldU32(
    std::shared_ptr<ripple::SLE>const & sle,
    ripple::SField const& field,
    std::uint32_t v
);
void setFieldU64(
    std::shared_ptr<ripple::SLE>const & sle,
    ripple::SField const& field,
    std::uint64_t v
);
void setFieldH160(
    std::shared_ptr<ripple::SLE>const & sle,
    ripple::SField const& field,
    ripple::uint160 const& v
);

void setFieldH256(
    std::shared_ptr<ripple::SLE>const & sle,
    ripple::SField const& field,
    ripple::uint256 const& v
);
void setFieldBlob(
    std::shared_ptr<ripple::SLE>const & sle,
    ripple::SField const& field,
    ripple::STBlob const& v
);

void makeFieldAbsent(
        std::shared_ptr<ripple::SLE>const & sle,
        ripple::SField const& field
        );

/*std::int32_t preflight1(ripple::PreflightContext const& ctx);
std::int32_t preflight2(ripple::PreflightContext const& ctx);*/

inline const ripple::STObject & upcast(const ripple::STTx &stTx) {
    return stTx;
}

inline std::shared_ptr<ripple::STObject> upcast_sle(const std::shared_ptr<ripple::SLE> &sle) {
    return sle;
}

using ConstSLE = ripple::SLE const;
inline ripple::STObject const& upcast_const_sle(ConstSLE const& sle) {
    return sle;
}

inline ripple::ReadView const& upcast_apply_view(ripple::ApplyView const& view) {
    return view;
}

constexpr std::uint32_t tfUniversalMask() {
    return ripple::tfUniversalMask;
}

constexpr ripple::SField const& sfRegularKey() {
    return ripple::sfRegularKey;
}

constexpr ripple::SField const& sfAccount() {
    return ripple::sfAccount;
}

constexpr ripple::SField const& sfSequence() {
    return ripple::sfSequence;
}

constexpr ripple::SField const& sfOwnerCount() {
    return ripple::sfOwnerCount;
}

constexpr ripple::SField const& sfOwnerNode() {
    return ripple::sfOwnerNode;
}

constexpr ripple::SField const& sfBalance() {
    return ripple::sfBalance;
}

constexpr ripple::SField const& sfFlags() {
    return ripple::sfFlags;
}

constexpr ripple::SField const& sfIssuer() {
    return ripple::sfIssuer;
}

constexpr ripple::SField const& sfTransferFee() {
    return ripple::sfTransferFee;
}

constexpr ripple::SField const& sfFee() {
    return ripple::sfFee;
}

constexpr ripple::SField const& sfAmount() {
    return ripple::sfAmount;
}

constexpr ripple::SField const& sfInvoiceId() {
    return ripple::sfInvoiceID;
}

constexpr ripple::SField const& sfDestination() {
    return ripple::sfDestination;
}

constexpr ripple::SField const& sfDestinationTag() {
    return ripple::sfDestinationTag;
}

constexpr ripple::SField const& sfPreviousPageMin() {
    return ripple::sfPreviousPageMin;
}

constexpr ripple::SField const& sfNextPageMin() {
    return ripple::sfNextPageMin;
}
/*constexpr ripple::SField const& sfTicketSequence() {
    return ripple::sfTicketSequence;
}*/

std::unique_ptr<std::string> toBase58(const ripple::AccountID& accountId);

/*void
foo(std::unique_ptr<std::vector<ripple::FakeSOElement>> vec);*/

//rust::Vec<ripple::FakeSOElement> getTxFormat();

// (1) Mayukha's code calls this (this has to be callable in the dylib)
//void getTxFormat2(std::vector<ripple::FakeSOElement> vec);
// (2) getTxFormat2 will call a Rust function over the bridge and get a rust::Vec and copy the values into the std::vector that
//    gets passed in

using OptionalSTVar = std::optional<ripple::detail::STVar>;
using OptionalUInt64 = std::optional<std::uint64_t>;

typedef const OptionalSTVar* (*parseLeafTypePtr)(
        ripple::SField const&,
        std::string const&,
        std::string const&,
        ripple::SField const*,
        Json::Value const&,
        Json::Value&);

struct STypeExport {
    int typeId;
    ripple::createNewSFieldPtr createPtr;
    parseLeafTypePtr parsePtr;
    ripple::constructSTypePtr constructPtr;
    ripple::constructSTypePtr2 constructPtr2;
};

using CreateNewSFieldPtr = ripple::SField const& (*)(int tid, int fv, const char* fn);
using ParseLeafTypeFnPtr = const OptionalSTVar* (*)(
ripple::SField const&,
std::string const&,
std::string const&,
ripple::SField const*,
Json::Value const&,
Json::Value&);

using STypeFromSITFnPtr = ripple::STBase* (*)(ripple::SerialIter&, ripple::SField const&);
using STypeFromSFieldFnPtr = ripple::STBase* (*)(ripple::SField const&);

void push_soelement(int field_code, ripple::SOEStyle style, std::vector<ripple::FakeSOElement>& vec);
void push_stype_export(int tid, CreateNewSFieldPtr createNewSFieldPtr, ParseLeafTypeFnPtr parseLeafTypeFn, STypeFromSITFnPtr sTypeFromSitFnPtr, STypeFromSFieldFnPtr sTypeFromSFieldFnPtr, std::vector<STypeExport>& vec);
void push_sfield_info(int tid, int fv, const char * txt_name, std::vector<ripple::SFieldInfo>& vec);

ripple::SField const& constructSField(int tid, int fv, const char* fn);

std::unique_ptr<OptionalSTVar> make_empty_stvar_opt() {
    OptionalSTVar ret;
    return std::make_unique<OptionalSTVar>(ret);
}

std::unique_ptr<OptionalSTVar> make_stvar(ripple::SField const& field, rust::Slice<const uint8_t> slice);

void bad_type(Json::Value& error, std::string const& json_name, std::string const& field_name);
void invalid_data(Json::Value& error, std::string const& json_name, std::string const& field_name);
std::unique_ptr<std::string> asString(Json::Value const& value);

std::unique_ptr<ripple::Buffer> getVLBuffer(ripple::SerialIter& sit);
std::unique_ptr<ripple::STPluginType> make_stype(ripple::SField const& field, std::unique_ptr<ripple::Buffer> buffer);
std::unique_ptr<ripple::STBase> make_empty_stype(ripple::SField const& field);
ripple::SField const& getSField(int type_id, int field_id);

std::shared_ptr<ripple::SLE> new_sle(ripple::Keylet const& k);

std::unique_ptr<std::optional<std::uint64_t>>
dir_insert(ripple::ApplyView& view, ripple::Keylet const& directory, ripple::Keylet const& key, ripple::AccountID const& account);
bool has_value(const std::unique_ptr<std::optional<std::uint64_t>> & optional);
std::uint64_t get_value(const std::unique_ptr<std::optional<std::uint64_t>> & optional);

bool opt_uint256_has_value(const std::unique_ptr<std::optional<ripple::uint256>> & optional);
ripple::uint256 opt_uint256_get_value(const std::unique_ptr<std::optional<ripple::uint256>> & optional);

using OptionalUint256 = std::optional<ripple::uint256>;
std::unique_ptr<OptionalUint256> apply_view_succ(ripple::ApplyView& applyView, ripple::Keylet const& key, ripple::Keylet const& last);
std::unique_ptr<OptionalUint256> read_view_succ(ripple::ReadView const& readView, ripple::Keylet const& key, ripple::Keylet const& last);

void
adjustOwnerCount(
    ripple::ApplyView& view,
    std::shared_ptr<ripple::SLE> const& sle,
    std::int32_t amount,
    beast::Journal const& j);

ripple::STBlob const& new_st_blob(ripple::SField const& field, std::uint8_t const* data, std::size_t size);

bool is_zero(ripple::STAmount const& amount) {
    return amount == beast::zero;
}

std::shared_ptr<ConstSLE> read(ripple::ReadView const& readView, ripple::Keylet const& k) {
    return readView.read(k);
}

bool st_amount_eq(ripple::STAmount const& amount1, ripple::STAmount const& amount2) {
    return amount1 == amount2;
}

bool st_amount_gt(ripple::STAmount const& amount1, ripple::STAmount const& amount2) {
    return amount1 > amount2;
}

std::unique_ptr<ripple::STArray> new_st_array() {
    return std::make_unique<ripple::STArray>();
}

ripple::STObject const& get_from_const_st_array(ripple::STArray const& array, std::size_t index);
std::unique_ptr<ripple::STObject> get_from_st_array(ripple::STArray const& array, std::size_t index);

std::unique_ptr<ripple::STObject> create_inner_object(ripple::SField const& field);

std::unique_ptr<ripple::STArray> peekFieldArray(std::shared_ptr<ripple::STObject> obj, ripple::SField const& field);
#endif //PLUGIN_TRANSACTOR_BLOBSTORE_H
