#ifndef RIPPLE_TEST_JTX_CFT_H_INCLUDED
#define RIPPLE_TEST_JTX_CFT_H_INCLUDED

#include <test/jtx/Account.h>

#include <ripple/protocol/UintTypes.h>

namespace ripple {
    namespace test {
        namespace jtx {

            namespace ballot {

                static const std::string DOCUMENT_URI_MAX = "did:xrpl:foo:12345678901234567890123456789012345678901234"
                                                            "567890123456789012345678901234567890123456789012345678901"
                                                            "234567890123456789012345678901234567890123456789012345678"
                                                            "901234567890123456789012345678901234567890123456789012345";

                static const std::string DOCUMENT_URI_TOO_LONG = "did:xrpl:foo:1234567890123456789012345678901234567890"
                                                                 "12345678901234567890123456789012345678901234567890123"
                                                                 "45678901234567890123456789012345678901234567890123456"
                                                                 "78901234567890123456789012345678901234567890123456789"
                                                                 "012345678901234567890123456789001234567891234";

                static std::string empty;

                /** Create a Ballot with default fields. */
                Json::Value
                create(jtx::Account const &account);

                /** Issue a Ballot with user-defined fields. */
                Json::Value
                create(
                        jtx::Account const &account,
                        std::uint32_t openTime,
                        std::uint32_t closeTime,
                        ripple::uint256 const &membershipNFTIssuer,
                        std::uint32_t membershipNFTTaxon,
                        ripple::uint256 const &ballotDocumentHash,
                        std::string const &ballotDocumentURI,
                        std::vector<uint256> const &ballotChoiceIDs
                );

                /** Delete a Ballot */
                Json::Value
                destroy(jtx::Account const &account, ripple::uint256 const &id);

            }  // namespace cft

        }  // namespace jtx
    }  // namespace test
}  // namespace ripple

#endif
