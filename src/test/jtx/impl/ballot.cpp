#include <test/jtx/ballot.h>
#include <ripple/protocol/TxFlags.h>
#include <ripple/protocol/jss.h>
#include <ripple/protocol/Indexes.h>
#include <ripple/protocol/digest.h>

/**
 * Test helper for creating ballots in unit tests.
 */
namespace ripple {
    namespace test {
        namespace jtx {

            namespace ballot {

                Json::Value
                create(jtx::Account const &account) {
                    Json::Value jv;
                    jv[sfTransactionType.jsonName] = jss::BallotCreate;
                    jv[sfFlags.jsonName] = tfUniversal;
                    jv[sfAccount.jsonName] = account.human();
                    jv[sfInitialVotes.jsonName] = (uint32_t) 1;
                    Json::Value choices(Json::arrayValue);
                    uint256 ballotChoiceID = sha512Half(0); // <-- arbitrary value 0 for hashing
                    choices.append(to_string(ballotChoiceID));
                    jv[sfBallotChoiceIDs.jsonName] = choices;
                    jv[sfBallotDocumentURI.jsonName] = strHex(std::string{DOCUMENT_URI_MAX});
                    jv[sfBallotDocumentHash.jsonName] = to_string(sha512Half(1)); // <-- arbitrary value 1 for hashing
                    jv[sfMembershipNFTIssuer.jsonName] = account.human();
                    jv[sfMembershipNFTTaxon.jsonName] = (uint32_t) 1;
                    return jv;
                }

                Json::Value
                create(
                        jtx::Account const &account,
                        std::uint32_t const openTime,
                        std::uint32_t const closeTime,
                        ripple::uint256 const &membershipNFTIssuer,
                        std::uint32_t const membershipNFTTaxon,
                        ripple::uint256 const &ballotDocumentHash,
                        std::string const &ballotDocumentURI,
                        std::vector<uint256> const &ballotChoiceIDs
                ) {
                    Json::Value jv;
                    jv[sfAccount.jsonName] = account.human();
                    jv[sfTransactionType.jsonName] = jss::BallotCreate;
                    jv[sfOpenTime.jsonName] = openTime;
                    jv[sfCloseTime.jsonName] = closeTime;
                    jv[sfMembershipNFTIssuer.jsonName] = to_string(membershipNFTIssuer);
                    jv[sfMembershipNFTTaxon.jsonName] = membershipNFTTaxon;
                    jv[sfBallotDocumentHash.jsonName] = to_string(ballotDocumentHash);
                    jv[sfBallotDocumentURI.jsonName] = strHex(ballotDocumentURI);

                    Json::Value choices(Json::arrayValue);
                    if (!ballotChoiceIDs.empty()) {
                        for (auto const &ballotChoiceID: ballotChoiceIDs) {
                            choices.append(to_string(ballotChoiceID));
                        }
                    }
                    jv[sfBallotChoiceIDs.jsonName] = choices;

                    return jv;
                }

                Json::Value
                destroy(jtx::Account const &account, ripple::uint256 const &id) {
                    Json::Value jv;
                    jv[sfAccount.jsonName] = account.human();
                    jv[sfBallotID.jsonName] = to_string(id);
                    jv[sfTransactionType.jsonName] = jss::BallotDelete;
                    return jv;
                }

                // TODO: Vote

//                Json::Value
//                authorize(
//                        jtx::Account const &account,
//                        ripple::uint256 const &issuanceID,
//                        std::optional<jtx::Account> const &holder) {
//                    Json::Value jv;
//                    jv[sfAccount.jsonName] = account.human();
//                    jv[sfTransactionType.jsonName] = jss::CFTokenAuthorize;
//                    jv[sfCFTokenIssuanceID.jsonName] = to_string(issuanceID);
//                    if (holder)
//                        jv[sfCFTokenHolder.jsonName] = holder->human();
//
//                    return jv;
//                }

//                Json::Value
//                set(jtx::Account const &account,
//                    ripple::uint256 const &issuanceID,
//                    std::optional<jtx::Account> const &holder) {
//                    Json::Value jv;
//                    jv[sfAccount.jsonName] = account.human();
//                    jv[sfTransactionType.jsonName] = jss::CFTokenIssuanceSet;
//                    jv[sfCFTokenIssuanceID.jsonName] = to_string(issuanceID);
//                    if (holder)
//                        jv[sfCFTokenHolder.jsonName] = holder->human();
//
//                    return jv;
//                }

            }  // namespace cft

        }  // namespace jtx
    }  // namespace test
}  // namespace ripple