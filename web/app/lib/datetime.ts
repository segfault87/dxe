import type { OccupiedSlot } from "../types/models/booking";

export function checkSlot(date: Date, slot: OccupiedSlot): boolean {
  const startTime = new Date(slot.bookingDate);
  const endTime = new Date(
    startTime.getTime() + slot.bookingHours * 60 * 60 * 1000,
  );

  return date >= startTime && date < endTime;
}

export function checkSlots(date: Date, slots: OccupiedSlot[]): boolean {
  if (slots.length === 0) {
    return false;
  }

  for (const slot of slots) {
    if (checkSlot(date, slot)) {
      return true;
    }
  }

  return false;
}

export function toUtcIso8601(date: Date): string {
  const year = date.getFullYear();
  const month = (date.getMonth() + 1).toString().padStart(2, "0");
  const day = date.getDate().toString().padStart(2, "0");
  const hours = date.getHours().toString().padStart(2, "0");
  const minutes = date.getMinutes().toString().padStart(2, "0");
  const seconds = date.getSeconds().toString().padStart(2, "0");
  const milliseconds = date.getMilliseconds().toString().padStart(3, "0");

  const offsetMinutes = date.getTimezoneOffset(); // Difference in minutes from UTC
  const offsetSign = offsetMinutes > 0 ? "-" : "+";
  const absOffsetMinutes = Math.abs(offsetMinutes);
  const offsetHours = Math.floor(absOffsetMinutes / 60)
    .toString()
    .padStart(2, "0");
  const offsetRemainingMinutes = (absOffsetMinutes % 60)
    .toString()
    .padStart(2, "0");

  return `${year}-${month}-${day}T${hours}:${minutes}:${seconds}.${milliseconds}${offsetSign}${offsetHours}:${offsetRemainingMinutes}`;
}
