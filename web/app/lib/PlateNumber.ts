const PLATE_NUMBER_REGEX = new RegExp(
  "^[0-9]{2,3}[가-힣]{1}[0-9]{4}$|^[가-힣]{2}[0-9]{1,2}[가-힣]{1}[0-9]{4}$",
);

export default function checkPlateNumber(plateNumber: string): boolean | null {
  if (plateNumber.length === 0) {
    return null;
  }

  return plateNumber.match(PLATE_NUMBER_REGEX) !== null;
}
