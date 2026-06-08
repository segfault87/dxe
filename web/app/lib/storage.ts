import type { AdhocReservationId } from "../types/models/base";

const TEMPORARY_RESERVATION_ID_KEY = "temporaryReservationId";

function getTemporaryReservationId(): AdhocReservationId | null {
  const result = window.localStorage.getItem(TEMPORARY_RESERVATION_ID_KEY);

  if (result === null) {
    return null;
  } else {
    return parseInt(result);
  }
}

function setTemporaryReservationId(id: AdhocReservationId) {
  window.localStorage.setItem(TEMPORARY_RESERVATION_ID_KEY, id.toString());
}

function clearTemporaryReservationId() {
  window.localStorage.removeItem(TEMPORARY_RESERVATION_ID_KEY);
}

export const TemporaryReservationIdStorage = {
  get: getTemporaryReservationId,
  set: setTemporaryReservationId,
  clear: clearTemporaryReservationId,
};
