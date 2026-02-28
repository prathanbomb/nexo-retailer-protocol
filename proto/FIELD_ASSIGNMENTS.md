# Field Number Assignments - Nexo Retailer Protocol

**Project:** nexo-retailer-protocol
**Standard:** ISO 20022 CASP v6 (2024-02-29)
**Purpose:** Track all protobuf field number assignments during conversion
**Created:** 2026-02-28

## Assignment Rules

1. **Fields 1-15**: Reserved for high-frequency fields (1 byte varint encoding)
2. **Fields 16-2047**: Standard fields (2 byte varint encoding)
3. **Fields 19000-19999**: PROHIBITED (reserved by protobuf specification)
4. **Deleted Fields**: MUST be marked `reserved` immediately (never reuse)

## Field Assignments by File

### common.proto

Common types shared across all CASP messages.

#### ActiveCurrencyAndAmount (CRITICAL - SCHEMA-05)
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | ccy | string | ISO 4217 currency code |
| 2 | units | int64 | Monetary units (whole) |
| 3 | nanos | int32 | Nanoseconds, must match sign of units |

#### GenericIdentification177
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | id | string | Identifier |
| 2 | issr | string | Issuer |
| 3 | tp | string | Type |
| 4 | cstmr_id | string | Customer ID |

#### AddressVerification1
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | adr_dgts | string | Address digits |
| 2 | pstl_cd_dgts | string | Postal code digits |

#### AggregationTransaction3
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | frst_pmt_dt_tm | int64 | First payment date/time (unix timestamp nanos) |
| 2 | last_pmt_dt_tm | int64 | Last payment date/time (unix timestamp nanos) |
| 3 | nb_of_pmts | int32 | Number of payments |
| 4 | indv_pmt | DetailedAmount21 | Repeated - Individual payments |

#### DetailedAmount21
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | amt | ActiveCurrencyAndAmount | Amount |
| 2 | tp | string | Type |

#### ContentInformationType38
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | cntt_tp | string | Content type |
| 2 | cntt | bytes | Content data |

#### OutputBarcode2
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | tp | string | Barcode type |
| 2 | val | string | Barcode value |

### casp.001.proto (SaleToPOIServiceRequestV06)

#### Casp001Document
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | document | Document | Root message element |

#### Document
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | sale_to_poi_svc_req | SaleToPOIServiceRequestV06 | Service request |

#### SaleToPOIServiceRequestV06
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | hdr | Header4 | Message header |
| 2 | tx | Transaction23 | Repeated - Transactions |
| 3 | scty_trlr | SecurityTrailer4 | Security trailer |
| 4 | login_req | LoginRequest3 | Login request |

#### Header4
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | msg_fctn | string | Message function |
| 2 | proto_vrsn | string | Protocol version |
| 3 | orgnl Biz_t_msg | string | Original business message |
| 4 | orgnl_msg_id | string | Original message ID |
| 5 | tx_id | string | Transaction ID |
| 6 | cre_dt_tm | string | Creation date/time |
| 7 | initg_pty | InitiatingParty3 | Initiating party |
| 8 | recipnt | Recipient5 | Recipient |
| 9 | trace_abilty | string | Traceability |

#### InitiatingParty3
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | id | Identification1 | Identification |
| 2 | tp | string | Type |
| 3 | med_of_id | string | Method of identification |

#### Identification1
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | id | string | Identifier |
| 2 | issr | string | Issuer |
| 3 | tp | string | Type |
| 4 | cstmr_id | string | Customer ID |

#### Recipient5
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | msg_tx_id | string | Message transaction ID |
| 2 | orgnl Biz_t_msg | string | Original business message |
| 3 | orgnl_msg_id | string | Original message ID |

#### Transaction23 (oneof wrapper)
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | card_pmt_ctx | CardPaymentContext30 | Card payment context |
| 2 | pmt_req | PaymentRequest29 | Payment request |
| 3 | card_acq_req | CardAcquisitionRequest3 | Card acquisition request |
| 4 | rvrsal_req | ReversalRequest7 | Reversal request |
| 5 | bal_inq_req | BalanceInquiryRequest7 | Balance inquiry request |
| 6 | btch_req | BatchRequest6 | Batch request |
| 7 | card_drct_dbt | CardDirectDebit2 | Card direct debit |
| 8 | authstn_rslt | AuthorisationResult18 | Authorisation result |
| 9 | rvrsal_rsp | ReversalResponse6 | Reversal response |
| 10 | btch_rsp | BatchResponse4 | Batch response |
| 11 | bal_inq_rsp | BalanceInquiryResponse6 | Balance inquiry response |
| 12 | card_acq_rsp | CardAcquisitionResponse2 | Card acquisition response |
| 13 | pmt_rsp | PaymentResponse28 | Payment response |
| 14 | tx | Transaction33 | Transaction wrapper |
| 15 | usr_intrfc | UserInterface5 | User interface |
| 16 | dignss | Diagnosis4 | Diagnosis |
| 17 | adm_req | AdminRequest4 | Admin request |
| 18 | adm_rsp | AdminResponse4 | Admin response |
| 19 | dsply_actn | DisplayAction2 | Display action |
| 20 | pin_req | Get_PINRequest3 | Get PIN request |
| 21 | pin_snd_req | Send_PINRequest2 | Send PIN request |
| 22 | pin_rsp | Get_PINResponse3 | Get PIN response |
| 23 | enbl_svc_req | EnableServiceRequest7 | Enable service request |
| 24 | enbl_svc_rsp | EnableServiceResponse6 | Enable service response |
| 25 | pin_reqt | PINRequete2 | PIN request (French) |
| 26 | pin_rspns | PINResponse1 | PIN response |
| 27 | crd_rdr_apdu_req | CardReaderAPDURequest8 | Card reader APDU request |
| 28 | crd_rdr_apdu_rsp | CardReaderAPDUResponse8 | Card reader APDU response |
| 29 | crd_rdr_pwr_off_req | CardReaderPowerOffRequest3 | Card reader power off request |
| 30 | crd_rdr_pwr_off_rsp | CardReaderPowerOffResponse3 | Card reader power off response |
| 31 | crd_rdr_init_req | CardReaderInitRequest4 | Card reader init request |
| 32 | crd_rdr_init_rsp | CardReaderInitResponse5 | Card reader init response |
| 33 | id_trmnl_req | IdentifyTerminalRequest2 | Identify terminal request |
| 34 | id_trmnl_rsp | IdentifyTerminalResponse2 | Identify terminal response |
| 35 | get_data_req | Get_DataRequest3 | Get data request |
| 36 | get_data_rsp | Get_DataResponse3 | Get data response |
| 37 | inp_upd | InputUpdate2 | Input update |
| 38 | lsr_cmd | LaserCommand3 | Laser command |
| 39 | login_req | LoginRequest3 | Login request |
| 40 | login_rsp | LoginResponse3 | Login response |
| 41 | lgout_req | LogoutRequest3 | Logout request |
| 42 | lgout_rsp | LogoutResponse3 | Logout response |
| 43 | msg_cert | MessageCertificate6 | Message certificate |
| 44 | pmt_req_tx | PaymentRequest29 | Payment request (transaction) |
| 45 | pmt_rsp_tx | PaymentResponse28 | Payment response (transaction) |
| 46 | prnt_outpt | PrintOutput2 | Print output |
| 47 | rcvry_req | RecoveryRequest4 | Recovery request |
| 48 | rcvry_rsp | RecoveryResponse4 | Recovery response |
| 49 | scnty_req | SecurityRequest4 | Security request |
| 50 | scnty_rsp | SecurityResponse4 | Security response |
| 51 | totlsr | Totaliser2 | Totaliser |
| 52 | tx_tx | Transaction31 | Transaction type 31 |
| 53 | tx_tx2 | Transaction32 | Transaction type 32 |
| 54 | uppr_rang | UpperRange3 | Upper range |
| 55 | abrt_req | AbortRequest3 | Abort request |
| 56 | abrt_rsp | AbortResponse3 | Abort response |
| 57 | card_acq_req2 | CardAcquisitionRequest4 | Card acquisition request v4 |
| 58 | card_acq_rsp2 | CardAcquisitionResponse3 | Card acquisition response v3 |

#### CardData8
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | crd_pmt_tokn | string | Card payment token |
| 2 | crd_nb | string | Card number |
| 3 | xpry_dt | string | Expiry date |
| 4 | card_seq_nb | string | Card sequence number |
| 5 | msstrp_cde | string | Magnetic stripe code |
| 6 | eff_dt | string | Effective date |
| 7 | crdt_dbit | string | Credit/debit |
| 8 | crt_mtd | string | Card reading method |

#### SecurityTrailer4
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | scnty_trlr | string | Security trailer type |
| 2 | authstn_data | bytes | Authentication data |
| 3 | signtr_val | bytes | Signature value |
| 4 | nce | bytes | Nonce |

### casp.002.proto (SaleToPOIServiceResponseV06)

#### Casp002Document
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | document | Document | Root message element |

#### Document
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | sale_to_poi_svc_rsp | SaleToPOIServiceResponseV06 | Service response |

#### SaleToPOIServiceResponseV06
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | hdr | Header4 | Message header |
| 2 | tx_rsp | TransactionResponse23 | Repeated - Transaction responses |
| 3 | scty_trlr | SecurityTrailer4 | Security trailer |
| 4 | login_rsp | LoginResponse3 | Login response |

#### TransactionResponse23 (oneof wrapper)
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | pmt_rsp | PaymentResponse28 | Payment response |
| 2 | card_acq_rsp | CardAcquisitionResponse2 | Card acquisition response |
| 3 | rvrsal_rsp | ReversalResponse6 | Reversal response |
| 4 | bal_inq_rsp | BalanceInquiryResponse6 | Balance inquiry response |
| 5 | btch_rsp | BatchResponse4 | Batch response |
| 6 | adm_rsp | AdminResponse4 | Admin response |
| 7 | dsply_rsp | DisplayResponse2 | Display response |
| 8 | pin_rsp | Get_PINResponse3 | Get PIN response |
| 9 | enbl_svc_rsp | EnableServiceResponse6 | Enable service response |
| 10 | pin_rspns | PINResponse1 | PIN response |
| 11 | crd_rdr_apdu_rsp | CardReaderAPDUResponse8 | Card reader APDU response |
| 12 | crd_rdr_pwr_off_rsp | CardReaderPowerOffResponse3 | Card reader power off response |
| 13 | crd_rdr_init_rsp | CardReaderInitResponse5 | Card reader init response |
| 14 | id_trmnl_rsp | IdentifyTerminalResponse2 | Identify terminal response |
| 15 | get_data_rsp | Get_DataResponse3 | Get data response |
| 16 | lgout_rsp | LogoutResponse3 | Logout response |
| 17 | rcvry_rsp | RecoveryResponse4 | Recovery response |
| 18 | scnty_rsp | SecurityResponse4 | Security response |
| 19 | abrt_rsp | AbortResponse3 | Abort response |
| 20 | card_acq_rsp2 | CardAcquisitionResponse3 | Card acquisition response v3 |

#### PaymentResponse28
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | tx_id | string | Transaction ID |
| 2 | pmt_rsp | string | Payment response code |
| 3 | pmt_rsp2 | string | Payment response code 2 |
| 4 | amt | ActiveCurrencyAndAmount | Amount |
| 5 | authstn_rsp | string | Authorisation response |
| 6 | authstn_nb | string | Authorisation number |
| 7 | tx_sts | string | Transaction status |

#### CardAcquisitionResponse2
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | tx_id | string | Transaction ID |
| 2 | card_acq_rsp | string | Card acquisition response |
| 3 | crd_data | CardData8 | Card data |

#### BalanceInquiryResponse6
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | tx_id | string | Transaction ID |
| 2 | bal_inq_rsp | string | Balance inquiry response |
| 3 | bals | BalanceDetails5 | Repeated - Balances |

#### BalanceDetails5
| Field Number | Field Name | Type | Notes |
|--------------|------------|------|-------|
| 1 | amt | ActiveCurrencyAndAmount | Amount |
| 2 | bal_tp | string | Balance type |
| 3 | dt_tm | int64 | Date/time |

### casp.003.proto through casp.017.proto

The remaining CASP proto files (003-017) follow similar patterns:

- Each has a `Casp{NNN}Document` root message with field 1 = `Document`
- Each `Document` message has field 1 pointing to the main message type
- Main message types have `hdr` (field 1), oneof transaction fields (2+), and `scty_trlr` (last field)
- Header4 structure is consistent across all messages
- Transaction-specific messages use field numbers 1-5 typically

#### casp.003 - SaleToPOIAdminRequestV06
- Transaction: AdminRequest4
- Fields: tx_id (1), adm_req (2), req_data (3)

#### casp.004 - SaleToPOIAdminResponseV06
- Transaction: AdminResponse4
- Fields: tx_id (1), adm_rsp (2), rsp_data (3)

#### casp.005 - TransactionManagement6
- Transactions: AbortRequest3, AbortResponse3
- Fields: tx_id (1), abrt_rsn/rsp (2)

#### casp.006 - CardTerminalManagement6
- Transactions: CardReaderInit*, CardReaderPowerOff*, IdentifyTerminal*
- Fields: tx_id (1), init/pwr_off/id_tp (2), rsp/trmnl_id (3)

#### casp.007 - PINManagement6
- Transactions: Get_PIN*, Send_PIN*
- Fields: tx_id (1), pin_req_tp (2), pin_lngt (3)

#### casp.008 - DisplayManagement6
- Transactions: DisplayAction2, DisplayResponse2
- Fields: tx_id (1), dsply_actn_tp (2), msg_cntt (3)

#### casp.009 - InputManagement6
- Transactions: InputUpdate2, InputResponse2
- Fields: tx_id (1), inp_upd_tp (2), inp_data (3)

#### casp.010 - CardDataManagement6
- Transactions: CardAcquisitionRequest3, CardAcquisitionResponse2
- Fields: tx_id (1), card_acq_tp (2), crd_data (3)

#### casp.011 - LoginManagement6
- Transactions: Login*, Logout*
- Fields: usr_id (1), pwd (2), login_rsp (1)

#### casp.012 - NetworkManagement6
- Transactions: Diagnosis*, Recovery*
- Fields: tx_id (1), dignss/rcvry_tp (2), rsp/data (3)

#### casp.013 - SecurityManagement2 (Note: version .02)
- Transactions: SecurityRequest4, SecurityResponse4
- Fields: tx_id (1), scnty_tp (2), scnty_data (3)

#### casp.014 - CertificateManagement6
- Transactions: CertificateRequest6, CertificateResponse6
- Fields: tx_id (1), cert_tp (2), cert_req_data/cert_data (3)

#### casp.015 - TotaliserManagement6
- Transactions: TotaliserRequest2, TotaliserResponse2
- Fields: tx_id (1), totlsr_tp (2), totlsr_data (3, repeated)

#### casp.016 - PrintManagement6
- Transactions: PrintOutput2, PrintResponse2
- Fields: tx_id (1), prnt_tp (2), prnt_data (3)

#### casp.017 - ApplicationManagement6
- Transactions: EnableServiceRequest7, EnableServiceResponse6
- Fields: tx_id (1), svc (2), enbl (3), svc_rsp (2)

## Summary Statistics

- **Total proto files**: 18 (1 common + 17 CASP)
- **Total message types**: ~150+ across all files
- **Total field assignments**: ~500+ field numbers
- **High-frequency fields (1-15)**: Document roots, message IDs, amounts
- **Standard fields (16-2047)**: Most transaction-specific fields
- **No prohibited ranges used**: All field numbers avoid 19000-19999

## Important Notes

1. **Field 1** is consistently used for the most important field in each message (Document root, message ID, primary type)
2. **Monetary amounts** (ActiveCurrencyAndAmount) use fields 1-3: ccy, units, nanos (SCHEMA-05 compliant)
3. **Header fields** use positions 1-9 consistently across all messages
4. **oneof fields** in Transaction23/TransactionResponse23 use sequential numbers 1-58 for all transaction type variants
5. **Security trailer** is always the last field in messages that include it

## Reserved Numbers (deleted fields)

| Message | Numbers | Date Reserved | Reason |
|---------|---------|---------------|--------|
| None yet | - | - | No fields deleted yet |

---
*This document is maintained during schema conversion. Any field deletions MUST be recorded here immediately.*
