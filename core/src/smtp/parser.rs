// Reacher - Email Verification
// Copyright (C) 2018-2022 Reacher

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Parse the SMTP responses to get information about the email address.

use super::error::SmtpError;
use async_smtp::{smtp::error::Error as AsyncSmtpError, EmailAddress};

/// is_invalid checks for SMTP responses meaning that the email is invalid,
/// i.e. that the mailbox doesn't exist.
pub fn is_invalid(e: &str, email: &EmailAddress) -> bool {
	// 550 Address rejected
	// 550 5.1.1 : Recipient address rejected
	// 550 5.1.1 : Recipient address rejected: User unknown in virtual alias table
	// 550 5.1.1 <EMAIL: Recipient address rejected: User unknown in relay recipient table
	e.contains("address rejected")
	// 550 5.1.1 : Unrouteable address
	|| e.contains("unrouteable")
	// 550 5.1.1 : The email account that you tried to reach does not exist
	|| e.contains("does not exist")
	// 550 invalid address
	// 550 User not local or invalid address – Relay denied
	|| e.contains("invalid address")
	// 5.1.1 Invalid email address
	|| e.contains("invalid email address")
	// 550 Invalid recipient
	|| e.contains("invalid recipient")
	|| e.contains("may not exist")
	|| e.contains("recipient invalid")
	// 550 5.1.1 : Recipient rejected
	|| e.contains("recipient rejected")
	// permanent: 5.1.1 Unknown recipient address
	|| e.contains("unknown recipient address")
	// 554 Unknown Recipient (#5.1.1) (on @parkwayhonda.com)
	|| e.contains("unknown recipient") 
	|| e.contains("undeliverable")
	// 550 User unknown
	// 550 5.1.1 <EMAIL> User unknown
	// 550 recipient address rejected: user unknown in local recipient table
	|| e.contains("user unknown")
	// 550 Unknown user
	|| e.contains("unknown user")
	// 5.1.1 Recipient unknown <EMAIL>
	|| e.contains("recipient unknown")
	// 550 5.1.1 No such user - pp
	// 550 No such user here
	|| e.contains("no such user")
	// permanent: 5.1.1 MXIN501 mailbox <EMAIL> unknown (on @virginmedia.com)
	|| e.contains(format!("mailbox {} unknown", email).as_str())
	// 550 5.1.1 : Mailbox not found
	// 550 Unknown address error ‘MAILBOX NOT FOUND’
	|| e.contains("mailbox not found")
	// 550 5.1.1 : Invalid mailbox
	|| e.contains("invalid mailbox")
	// 550 5.1.1 Sorry, no mailbox here by that name
	|| e.contains("no mailbox")
	// 5.2.0 No such mailbox
	|| e.contains("no such mailbox")
	// 550 Requested action not taken: mailbox unavailable
	|| e.contains("mailbox unavailable")
	// 5.5.0 Requested actions not taken as the mailbox is unavailable (on @etu.uca.fr)
	|| e.contains("mailbox is unavailable")
	// 550 5.1.1 Is not a valid mailbox
	|| e.contains("not a valid mailbox")
	// No such recipient here
	|| e.contains("no such recipient")
	// 554 delivery error: This user doesn’t have an account
	|| e.contains("have an account")
	// permanent: Unknown local part <USER> in <USER@flabeg.com> (on @flabeg.com)
	|| e.contains("unknown local part")
	// 5.1.1 RCP-P1 Domain facebook.com no longer available https://www.facebook.com/postmaster/response_codes?ip=3.80.111.155#RCP-P1
	|| e.contains("no longer available")
	// permanent: RCPT (<EMAIL>) dosn't exist (on @hgy.ooo, @stigpods.com.cn)
	|| e.contains("dosn't exist") // sic! typo is intentional
	// 5.1.1 <EMAIL>: Email address could not be found, or was misspelled (G8) (on @biotech-calendar.com, @invoicefactoring.com)
	|| e.contains("could not be found") 
	// No such person at this address (on @aconsa.com.mx)
	|| e.contains("no such person")
	// Callout verification failed: 550 No Such User Here (on @medipro.co.uk)
	|| e.contains("no such user")
	// 5.1.1 <EMAIL> Address Error (on @lucidity.co.za)
	|| e.contains("address error")
	// E-mail address is not handled by this system (on @kaimayfair.co.uk)
	|| e.contains("address is not handled")
}

/// Check that the mailbox has a full inbox.
pub fn is_full_inbox(e: &str) -> bool {
	e.contains("insufficient")
	// https://answers.microsoft.com/en-us/outlook_com/forum/all/how-do-i-interpret-the-delivery-failure-message/2f1bf9c0-8b03-4f8f-aacc-5f6ba60a73f3
	|| e.contains("mailbox full")
	// https://answers.microsoft.com/en-us/outlook_com/forum/all/how-do-i-interpret-the-delivery-failure-message/2f1bf9c0-8b03-4f8f-aacc-5f6ba60a73f3
	|| e.contains("quote exceeded")
	|| e.contains("over quota")
	// 550 user has too many messages on the server
	|| e.contains("too many messages")
}

/// Check if the email account has been disabled or blocked by the email
/// provider.
pub fn is_disabled_account(e: &str) -> bool {
	// 554 The email account that you tried to reach is disabled. Learn more at https://support.google.com/mail/?p=DisabledUser"
	e.contains("disabled")
	// 554 delivery error: Sorry your message to <EMAIL> cannot be delivered. This account has been disabled or discontinued
 || e.contains("discontinued")
}

/// Check if the error is an IO "incomplete" error.
pub fn is_err_io_errors(e: &SmtpError) -> bool {
	match e {
		SmtpError::SmtpError(AsyncSmtpError::Io(err)) => err.to_string() == "incomplete",
		_ => false,
	}
}

/// Check if the IP is blacklisted.
pub fn is_err_ip_blacklisted(e: &SmtpError) -> bool {
	let e = match e {
		SmtpError::SmtpError(AsyncSmtpError::Transient(r) | AsyncSmtpError::Permanent(r)) => {
			// TODO We can use .to_string() after:
			// https://github.com/async-email/async-smtp/pull/53
			r.message.join("; ").to_lowercase()
		}
		_ => {
			return false;
		}
	};

	// Permanent errors

	// 5.7.1 IP address blacklisted by recipient
	// 5.7.1 Service unavailable; Client host [147.75.45.223] is blacklisted. Visit https://www.sophos.com/en-us/threat-center/ip-lookup.aspx?ip=147.75.45.223 to request delisting
	// 5.3.0 <EMAIL>... Mail from 147.75.45.223 rejected by Abusix blacklist (on @helsinki.fi)
	e.contains("blacklist")
	// Rejected because 23.129.64.213 is in a black list at b.barracudacentral.org
	|| e.contains("black list")
	// 5.7.1 Recipient not authorized, your IP has been found on a block list
	// gmx.net (mxgmx117) Nemesis ESMTP Service not available; No SMTP service; IP address is block listed.; For explanation visit https://www.gmx.net/mail/senderguidelines?c=bl (on @gmx.net, @web.de)
	|| e.contains("block list")
	// Unable to add <EMAIL> because host 23.129.64.184 is listed on zen.spamhaus.org
	// 5.7.1 Service unavailable, Client host [23.129.64.184] blocked using Spamhaus.
	// 5.7.1 Email cannot be delivered. Reason: Email detected as Spam by spam filters.
	|| e.contains("spam")
	// host 23.129.64.216 is listed at combined.mail.abusix.zone (127.0.0.12,
	|| e.contains("abusix")
	// 5.7.1 Relaying denied. IP name possibly forged [45.154.35.252]
	// 5.7.1 Relaying denied: You must check for new mail before sending mail. [23.129.64.216]
	|| e.contains("relaying denied")
	// 5.7.1 <unknown[23.129.64.100]>: Client host rejected: Access denied
	|| e.contains("access denied")
	// sorry, mail from your location [5.79.109.48] is administratively denied (#5.7.1)
	|| e.contains("administratively denied")
	// 5.7.606 Access denied, banned sending IP [23.129.64.216]
	|| e.contains("banned")
	// Blocked - see https://ipcheck.proofpoint.com/?ip=23.129.64.192
	// 5.7.1 Mail from 23.129.64.183 has been blocked by Trend Micro Email Reputation Service.
	|| e.contains("blocked")
	// Connection rejected by policy [7.3] 38206, please visit https://support.symantec.com/en_US/article.TECH246726.html for more details about this error message.
	|| e.contains("connection rejected")
	// csi.mimecast.org Poor Reputation Sender. - https://community.mimecast.com/docs/DOC-1369#550 [6ATVl4DjOvSA6XNsWGoUFw.us31]
	// Your access to this mail system has been rejected due to the sending MTA\'s poor reputation. If you believe that this failure is in error, please contact the intended recipient via alternate means.
	|| e.contains("poor reputation")
	// JunkMail rejected - (gmail.com) [193.218.118.140]:46615 is in an RBL: http://www.barracudanetworks.com/reputation/?pr=1&ip=193.218.118.140
	|| e.contains("junkmail")
	// mailfi01.lmco.com ESMTP 550 5.7.0  Mail from 18.234.87.196 refused by Proofpoint Reputation Services.  SENDER please see and take action: https://support.proofpoint.com/dnsbl-lookup.cgi?18.234.87.196" (on @lmco.com)
	|| e.contains("refused by proofpoint")
	// resimta-h1p-037598.sys.comcast.net resimta-h1p-037598.sys.comcast.net 5.135.185.166 found on one or more DNSBLs, see http://postmaster.comcast.net/smtp-error-codes.php#BL000001 (on @comcast.net)
	|| e.contains("dnsbl")
	// smtp-fw-9107.amazon.com; SBRS score too low: http://www.senderbase.org/ (on @amazon.com)
	|| e.contains("sbrs score too low")
	// https://www.spamhaus.org/sbl/query/SBLCSShttps://www.spamhaus.org/query/ip/3.238.201.74 (on @knollridges.com.ph)
	|| e.contains("spamhaus")

    // Transient errors

	// Blocked - see https://www.spamcop.net/bl.shtml?23.129.64.211
	|| e.contains("blocked")
	// 4.7.1 <EMAIL>: Relay access denied
	|| e.contains("access denied")
	// relay not permitted!
	|| e.contains("relay not permitted")
	// 23.129.64.216 is not yet authorized to deliver mail from
	|| e.contains("not yet authorized")
}

/// Check if the IP needs a reverse DNS.
pub fn is_err_needs_rdns(e: &SmtpError) -> bool {
	let e = match e {
		SmtpError::SmtpError(AsyncSmtpError::Transient(r) | AsyncSmtpError::Permanent(r)) => {
			// TODO We can use .to_string() after:
			// https://github.com/async-email/async-smtp/pull/53
			r.message.join("; ").to_lowercase()
		}
		_ => {
			return false;
		}
	};

	// 4.7.25 Client host rejected: cannot find your hostname, [147.75.45.223]
	// 4.7.1 Client host rejected: cannot find your reverse hostname, [147.75.45.223]
	// 5.7.1 Client host rejected: cannot find your reverse hostname, [23.129.64.184]
	e.contains("cannot find your reverse hostname")
	// You dont seem to have a reverse dns entry. Come back later. You are greylisted for 20 minutes. See http://www.fsf.org/about/systems/greylisting
	|| e.contains("reverse dns entry")
}

#[cfg(test)]
mod tests {

	use super::{is_err_ip_blacklisted, is_invalid};
	use crate::SmtpError::SmtpError;
	use async_smtp::{
		smtp::error::Error,
		smtp::response::{Category, Code, Detail, Response, Severity},
		EmailAddress,
	};
	use std::str::FromStr;

	#[test]
	fn test_is_invalid() {
		let email = EmailAddress::from_str("foo@bar.baz").unwrap();

		assert_eq!(
			is_invalid(
				"554 5.7.1 <mta.voipdir.net[]>: Client host rejected: Access denied",
				&email
			),
			false
		);

		assert_eq!(
			is_invalid("RCPT (***@stigpods.com.cn) dosn't exist", &email),
			true
		);

		assert_eq!(
			is_invalid(
				"permanent: 5.1.1 MXIN501 mailbox foo@bar.baz unknown (on @virginmedia.com)",
				&email
			),
			true
		);
	}

	#[test]
	fn test_is_err_ip_blacklisted() {
		let err = Error::Permanent(Response::new(
			Code::new(
				Severity::PermanentNegativeCompletion,
				Category::Information,
				Detail::Zero,
			),
			vec![
				"gmx.net (mxgmx117) Nemesis ESMTP Service not available".to_string(),
				"No SMTP service".to_string(),
				"IP address is block listed.".to_string(),
				"For explanation visit https://www.gmx.net/mail/senderguidelines?c=bl".to_string(),
			],
		));

		assert!(is_err_ip_blacklisted(&SmtpError(err)))
	}
}
