import "./AttachmentSelection.css";
import type { BookingAttachmentId } from "../types/models/base";
import type { BookingAttachment } from "../types/models/booking";

export interface AttachmentSelectionProps {
  attachments: BookingAttachment[];
  unavailableAttachmentIds: BookingAttachmentId[];
  // Attachments the booking already has; shown as checked and can't be toggled
  includedAttachmentIds?: BookingAttachmentId[];

  selectedAttachmentIds: BookingAttachmentId[];
  onSelectAttachmentIds: (attachmentIds: BookingAttachmentId[]) => void;
}

export default function AttachmentSelection({
  attachments,
  unavailableAttachmentIds,
  includedAttachmentIds = [],
  selectedAttachmentIds,
  onSelectAttachmentIds,
}: AttachmentSelectionProps) {
  const toggle = (attachmentId: BookingAttachmentId, checked: boolean) => {
    if (checked) {
      onSelectAttachmentIds([...selectedAttachmentIds, attachmentId]);
    } else {
      onSelectAttachmentIds(
        selectedAttachmentIds.filter((v) => v !== attachmentId),
      );
    }
  };

  return (
    <ul className="attachment-selection">
      {attachments.map((attachment) => {
        const isIncluded = includedAttachmentIds.includes(attachment.id);
        const isUnavailable =
          !isIncluded && unavailableAttachmentIds.includes(attachment.id);

        let className;
        if (isIncluded) {
          className = "included";
        } else if (isUnavailable) {
          className = "unavailable";
        }

        return (
          <li key={attachment.id} className={className}>
            <input
              type="checkbox"
              checked={
                isIncluded ||
                (!isUnavailable && selectedAttachmentIds.includes(attachment.id))
              }
              disabled={isIncluded || isUnavailable}
              onChange={(e) => toggle(attachment.id, e.target.checked)}
              id={`attachment-${attachment.id}`}
            />
            <label htmlFor={`attachment-${attachment.id}`}>
              <img
                src={`/assets/attachments/${attachment.id}.png`}
                alt={attachment.name}
              />
              <span className="name">{attachment.name}</span>
              <span className="category">
                {attachment.category}
                {isIncluded ? " · 예약에 포함됨" : null}
              </span>
            </label>
          </li>
        );
      })}
    </ul>
  );
}

export function AttachmentList({
  attachments,
}: {
  attachments: BookingAttachment[];
}) {
  return (
    <ul className="attachment-selection readonly">
      {attachments.map((attachment) => (
        <li key={attachment.id}>
          <label>
            <img
              src={`/assets/attachments/${attachment.id}.png`}
              alt={attachment.name}
            />
            <span className="name">{attachment.name}</span>
            <span className="category">{attachment.category}</span>
          </label>
        </li>
      ))}
    </ul>
  );
}
