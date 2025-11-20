const PHONE_NUMBER_REGEX = new RegExp("^010-?[0-9]{4}-?[0-9]{4}$");

export function validatePhoneNumber(phoneNumber: string): boolean | null {
  if (phoneNumber.length === 0) {
    return null;
  }

  return phoneNumber.match(PHONE_NUMBER_REGEX) !== null;
}
