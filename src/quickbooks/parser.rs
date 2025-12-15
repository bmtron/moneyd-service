use std::io::{BufRead, BufReader};

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::utils::globalutil::parse_ofx_date;
use crate::utils::transactiontransporter::TransactionTransport;

const CREDIT_TYPE_CODE: i32 = 20;
const DEBIT_TYPE_CODE: i32 = 10;

#[derive(Debug)]
pub struct TempTranFromXml {
    transaction_type: String,
    date_posted: String,
    transaction_amount: String,
    refnum: String,
    name: String,
    memo: String,
}

impl TempTranFromXml {
    pub fn new() -> Self {
        Self {
            transaction_type: String::new(),
            date_posted: String::new(),
            transaction_amount: String::new(),
            refnum: String::new(),
            name: String::new(),
            memo: String::new(),
        }
    }

    pub fn to_transport(&self) -> TransactionTransport {
        let amount = self.transaction_amount.replace(".", "");
        let amount = amount.replace("-", "");
        let val = match amount.parse::<i32>() {
            Ok(v) => v,
            Err(e) => {
                println!("Invalid: {}", self.transaction_amount);
                0
            }
        };

        let date = parse_ofx_date(self.date_posted.as_str()).unwrap();

        TransactionTransport {
            statement_id: None,
            description: self.memo.clone(),
            amount: val,
            transaction_date: date,
            refnum: self.refnum.clone(),
            transaction_type_lookup_code: match self.transaction_type.as_str() {
                "DEBIT" => DEBIT_TYPE_CODE,
                "CREDIT" | "DIRECTDEP" => CREDIT_TYPE_CODE,
                _ => DEBIT_TYPE_CODE,
            },
        }
    }
}

pub fn parse_ofx_with_fallback(file_content: &str, file_name: &str) -> Vec<TempTranFromXml> {
    match parse_as_xml(&file_content.to_string()) {
        Ok(txns) if !txns.is_empty() => {
            println!("Parsed as OFX v2");
            return txns;
        }
        _ => {}
    }

    match parse_as_sgml(&file_content.to_string()) {
        Ok(txns) if !txns.is_empty() => {
            println!("Parsed as OFX v1");
            return txns;
        }
        _ => {}
    }

    match parse_as_sgml_on_one_line(&file_content.to_string()) {
        Ok(txns) if !txns.is_empty() => {
            println!("Parsed as OFX v1 that was all on one line");
            return txns;
        }
        _ => {}
    }

    panic!("Could not parse file {} as either OFX v1 or v2", file_name);
}

pub fn parse_as_xml(
    file_content: &String,
) -> Result<Vec<TempTranFromXml>, Box<dyn std::error::Error>> {
    println!("{}", "parsing as xml...");
    let mut x_reader = Reader::from_str(&file_content.as_str());
    x_reader.config_mut().trim_text(true);

    let mut buf: Vec<u8> = Vec::new();
    let mut txns: Vec<TempTranFromXml> = Vec::new();

    let mut curr_tag: String = String::new();

    let mut temp_tran_holder: Option<TempTranFromXml> = None;
    loop {
        match x_reader.read_event_into(&mut buf) {
            Err(e) => return Err(Box::new(e)),
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"STMTTRN" => {
                    if temp_tran_holder.is_some() {
                        txns.push(temp_tran_holder.expect("Holder was None somehow."));
                        temp_tran_holder = Some(TempTranFromXml {
                            transaction_type: String::new(),
                            date_posted: String::new(),
                            transaction_amount: String::new(),
                            refnum: String::new(),
                            name: String::new(),
                            memo: String::new(),
                        })
                    }
                }
                _ => (),
            },
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"STMTTRN" => {
                    if temp_tran_holder.is_none() {
                        temp_tran_holder = Some(TempTranFromXml {
                            transaction_type: String::new(),
                            date_posted: String::new(),
                            transaction_amount: String::new(),
                            refnum: String::new(),
                            name: String::new(),
                            memo: String::new(),
                        })
                    }
                }
                b"TRNTYPE" | b"DTPOSTED" | b"TRNAMT" | b"REFNUM" | b"NAME" | b"MEMO" | b"FITID" => {
                    curr_tag = match String::from_utf8(e.name().as_ref().to_owned()) {
                        Ok(s) => s,
                        Err(e) => {
                            println!("Failed to convert");
                            String::new()
                        }
                    }
                }
                _ => (),
            },
            Ok(Event::Text(e)) => match curr_tag.as_str() {
                "TRNTYPE" => match temp_tran_holder {
                    Some(ref mut t) => t.transaction_type = e.decode().unwrap().into_owned(),
                    _ => (),
                },
                "DTPOSTED" => match temp_tran_holder {
                    Some(ref mut t) => t.date_posted = e.decode().unwrap().into_owned(),
                    _ => (),
                },
                "TRNAMT" => match temp_tran_holder {
                    Some(ref mut t) => t.transaction_amount = e.decode().unwrap().into_owned(),
                    _ => (),
                },
                "REFNUM" => match temp_tran_holder {
                    Some(ref mut t) => t.refnum = e.decode().unwrap().into_owned(),
                    _ => (),
                },
                "NAME" => match temp_tran_holder {
                    Some(ref mut t) => t.name = e.decode().unwrap().into_owned(),
                    _ => (),
                },
                "MEMO" => match temp_tran_holder {
                    Some(ref mut t) => t.memo = e.decode().unwrap().into_owned(),
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }
        buf.clear();
    }

    Ok(txns)
}

pub fn parse_as_sgml_on_one_line(
    mut file_content: &String,
) -> Result<Vec<TempTranFromXml>, Box<dyn std::error::Error>> {
    let replaced_file_content = &file_content.replace("<", "\n<");
    parse_as_sgml(&replaced_file_content)
}
pub fn parse_as_sgml(
    file_content: &String,
) -> Result<Vec<TempTranFromXml>, Box<dyn std::error::Error>> {
    let reader = BufReader::new(file_content.as_bytes());
    let mut txns: Vec<TempTranFromXml> = Vec::new();
    let mut in_transaction = false;
    let mut current_txn = TempTranFromXml::new();

    for line in reader.lines() {
        let line = line.unwrap().trim().to_string();
        println!("line: {}", line);
        if line.is_empty() {
            continue;
        }

        if line.starts_with("<STMTTRN>") {
            in_transaction = true;
            current_txn = TempTranFromXml::new();
            continue;
        }

        if line.starts_with("</STMTTRN>") {
            in_transaction = false;
            txns.push(current_txn);
            current_txn = TempTranFromXml::new();
            continue;
        }

        if in_transaction && line.starts_with('<') {
            if let Some((tag, value)) = parse_sgml_line(&line) {
                match tag.as_str() {
                    "TRNTYPE" => current_txn.transaction_type = value,
                    "DTPOSTED" => current_txn.date_posted = value,
                    "TRNAMT" => current_txn.transaction_amount = value,
                    "FITID" | "REFNUM" => current_txn.refnum = value,
                    "NAME" => current_txn.name = value,
                    "MEMO" => current_txn.memo = value,
                    _ => {}
                }
            }
        }
    }

    Ok(txns)
}

fn parse_sgml_line(line: &String) -> Option<(String, String)> {
    if !line.starts_with('<') {
        return None;
    }
    let tag_end = line.find('>')?;
    let tag = line[1..tag_end].to_string();
    let rest = &line[tag_end + 1..];

    let value = if let Some(next_tag) = rest.find('<') {
        rest[..next_tag].trim().to_string()
    } else {
        rest.trim().to_string()
    };

    Some((tag, value))
}

#[cfg(test)]
mod tests {
    use super::*;
    const V1_SMGL_DATA: &str = r#"OFXHEADER:100
DATA:OFXSGML
VERSION:102
SECURITY:NONE
ENCODING:USASCII
CHARSET:1252
COMPRESSION:NONE
OLDFILEUID:NONE
NEWFILEUID:NONE

<OFX>
<SIGNONMSGSRSV1>
<SONRS>
<STATUS>
<CODE>0
<SEVERITY>INFO
</STATUS>
<DTSERVER>20251213120000
<LANGUAGE>ENG
</SONRS>
</SIGNONMSGSRSV1>
<BANKMSGSRSV1>
<STMTTRNRS>
<TRNUID>1 
<STATUS>
<CODE>0
<SEVERITY>INFO
</STATUS>
<STMTRS>
<CURDEF>USD
<BANKACCTFROM>
<BANKID>0
<ACCTID>0
<ACCTTYPE>FAKE
</BANKACCTFROM>
<BANKTRANLIST>
<DTSTART>20240613120000 
<DTEND>20251213120000 
<STMTTRN>
<TRNTYPE>DEBIT
<DTPOSTED>20251212120000
<TRNAMT>-999.99
<FITID>1
<NAME>Payment Place
<MEMO>Preauthorized Debit
</STMTTRN>
<STMTTRN>
<TRNTYPE>DEBIT
<DTPOSTED>20251212120000
<TRNAMT>-1.00
<FITID>2
<NAME>Payment Place
<MEMO>Preauthorized Debit
</STMTTRN>"#;

    const V2_XML_DATA: &str = r#"<?xml version="1.0" standalone="no"?><?OFX OFXHEADER="200" VERSION="202" SECURITY="NONE" OLDFILEUID="NONE" NEWFILEUID="NONE"?>
<OFX>
    <SIGNONMSGSRSV1>
        <SONRS>
            <STATUS>
                <CODE>0</CODE>
                <SEVERITY>INFO</SEVERITY>
                <MESSAGE>Login Successful!</MESSAGE>
            </STATUS>
            <DTSERVER>20251124000000.000[-7:MST]</DTSERVER>
            <LANGUAGE>ENG</LANGUAGE>
            <FI>
                <ORG>FAKE</ORG>
                <FID>1</FID>
            </FI>
            <INTU.BID>1</INTU.BID>
        </SONRS>
    </SIGNONMSGSRSV1>
    <CREDITCARDMSGSRSV1>
        <CCSTMTTRNRS>
            <TRNUID>0</TRNUID>
            <STATUS>
                <CODE>0</CODE>
                <SEVERITY>INFO</SEVERITY>
            </STATUS>
            <CCSTMTRS>
                <CURDEF>USD</CURDEF>
                <CCACCTFROM>
                    <ACCTID>0</ACCTID>
                </CCACCTFROM>
                <BANKTRANLIST>
                    <DTSTART>20251023000000.000[-7:MST]</DTSTART>
                    <DTEND>20251121000000.000[-7:MST]</DTEND>
                    <STMTTRN>
                        <TRNTYPE>DEBIT</TRNTYPE>
                        <DTPOSTED>20251120000000.000[-7:MST]</DTPOSTED>
                        <TRNAMT>-10.01</TRNAMT>
                        <FITID>1</FITID>
                        <REFNUM>1</REFNUM>
                        <NAME>Transaction 1 Name</NAME>
                        <MEMO>Transaction 1 Memo</MEMO>
                    </STMTTRN>
                    <STMTTRN>
                        <TRNTYPE>DEBIT</TRNTYPE>
                        <DTPOSTED>20251119000000.000[-7:MST]</DTPOSTED>
                        <TRNAMT>-5.01</TRNAMT>
                        <FITID>2</FITID>
                        <REFNUM>2</REFNUM>
                        <NAME>Transaction 2 Name</NAME>
                        <MEMO>Transaction 2 Memo</MEMO>
                    </STMTTRN>
                </BANKTRANLIST>
            </CCSTMTRS>
        </CCSTMTTRNRS>
    </CREDITCARDMSGSRSV1>
</OFX>"#;

    const ONE_LINE_TEST_DATA: &str = r#"OFXHEADER:100
DATA:OFXSGML
VERSION:102
SECURITY:NONE
ENCODING:USASCII
CHARSET:1252
COMPRESSION:NONE
OLDFILEUID:NONE
NEWFILEUID:NONE

<OFX><SIGNONMSGSRSV1><SONRS><STATUS><CODE>0<SEVERITY>INFO</STATUS><DTSERVER>20251215120000[0:GMT]<LANGUAGE>ENG<FI><ORG>Dummy<FID>000</FI></SONRS></SIGNONMSGSRSV1><CREDITCARDMSGSRSV1><CCSTMTTRNRS><TRNUID>0<STATUS><CODE>0<SEVERITY>INFO</STATUS><CCSTMTRS><CURDEF>USD<CCACCTFROM><ACCTID>00-test</CCACCTFROM><BANKTRANLIST><DTSTART>20251101120000[0:GMT]<DTEND>20251130120000[0:GMT]<STMTTRN><TRNTYPE>DEBIT<DTPOSTED>20251129120000[0:GMT]<TRNAMT>-0.92<FITID>test-123<NAME>Test Transaction</STMTTRN></BANKTRANLIST><LEDGERBAL><BALAMT>-1234.56<DTASOF>20251130120000[0:GMT]</LEDGERBAL><AVAILBAL><BALAMT>9999.99DTASOF>20251130120000[0:GMT]</AVAILBAL></CCSTMTRS></CCSTMTTRNRS></CREDITCARDMSGSRSV1></OFX>"#;

    #[test]
    fn test_parse_smgl_as_smgl() {
        let res = parse_as_sgml(&V1_SMGL_DATA.to_string());
        let unwrapped = res.unwrap();
        assert_eq!(unwrapped.len(), 2);
        let first_res = &unwrapped
            .get(0)
            .expect("First value is none. This is wrong.");
        let second_res = &unwrapped
            .get(1)
            .expect("Second value is none. This is wrong.");
        assert_eq!(first_res.memo, r#"Preauthorized Debit"#);
        assert_eq!(first_res.transaction_amount, r#"-999.99"#);
        assert_eq!(first_res.refnum, r#"1"#);
        assert_eq!(second_res.memo, r#"Preauthorized Debit"#);
        assert_eq!(second_res.transaction_amount, r#"-1.00"#);
        assert_eq!(second_res.refnum, r#"2"#);
    }
    #[test]
    fn test_parse_one_line_sgml() {
        let res = parse_as_sgml_on_one_line(&ONE_LINE_TEST_DATA.to_string());
        let unwrapped = res.unwrap();
        let first_res = &unwrapped
            .get(0)
            .expect("First value is none. This is wrong.");
        assert_eq!(first_res.name, r#"Test Transaction"#);
        assert_eq!(first_res.transaction_amount, r#"-0.92"#);
        assert_eq!(first_res.refnum, r#"test-123"#);
    }
    #[test]
    fn test_parse_smgl_as_xml() {
        let res = parse_as_xml(&V1_SMGL_DATA.to_string());
        let unwrapped = match res {
            Ok(s) => s,
            Err(_) => Vec::new(),
        };

        assert_eq!(unwrapped.len(), 0);
    }

    #[test]
    fn test_parse_xml_as_xml() {
        let res = parse_as_xml(&V2_XML_DATA.to_string());
        let unwrapped = match res {
            Ok(r) => r,
            Err(e) => {
                println!("{:?}", e);
                Vec::new()
            }
        };
        assert_eq!(unwrapped.len(), 2);
        let first_res = &unwrapped
            .get(0)
            .expect("First value is none. This is wrong.");
        let second_res = &unwrapped
            .get(1)
            .expect("Second value is none. This is wrong.");

        assert_eq!(first_res.memo, r#"Transaction 1 Memo"#);
        assert_eq!(first_res.transaction_amount, r#"-10.01"#);
        assert_eq!(first_res.refnum, r#"1"#);
        assert_eq!(second_res.memo, r#"Transaction 2 Memo"#);
        assert_eq!(second_res.transaction_amount, r#"-5.01"#);
        assert_eq!(second_res.refnum, r#"2"#);
    }

    #[test]
    fn test_parse_ofx_with_smgl() {
        let result = parse_ofx_with_fallback(&V1_SMGL_DATA, "dummy_file");

        assert_eq!(result.len(), 2);
        let first_res = &result.get(0).expect("First value is none. This is wrong.");
        let second_res = &result.get(1).expect("Second value is none. This is wrong.");
        assert_eq!(first_res.memo, r#"Preauthorized Debit"#);
        assert_eq!(first_res.transaction_amount, r#"-999.99"#);
        assert_eq!(first_res.refnum, r#"1"#);
        assert_eq!(second_res.memo, r#"Preauthorized Debit"#);
        assert_eq!(second_res.transaction_amount, r#"-1.00"#);
        assert_eq!(second_res.refnum, r#"2"#);
    }

    #[test]
    fn test_parse_ofx_with_xml() {
        let result = parse_ofx_with_fallback(&V2_XML_DATA, "dummy_file");

        assert_eq!(result.len(), 2);
        let first_res = &result.get(0).expect("First value is none. This is wrong.");
        let second_res = &result.get(1).expect("Second value is none. This is wrong.");

        assert_eq!(first_res.memo, r#"Transaction 1 Memo"#);
        assert_eq!(first_res.transaction_amount, r#"-10.01"#);
        assert_eq!(first_res.refnum, r#"1"#);
        assert_eq!(second_res.memo, r#"Transaction 2 Memo"#);
        assert_eq!(second_res.transaction_amount, r#"-5.01"#);
        assert_eq!(second_res.refnum, r#"2"#);
    }
}
