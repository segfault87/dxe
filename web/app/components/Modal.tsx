import ReactModal from "react-modal";

import "./Modal.css";

export interface ModalProps {
  isOpen: boolean;
  close: () => void;
}

const ModalStyle: ReactModal.Styles = {
  overlay: {
    position: "fixed",
    top: "0",
    left: "0",
    width: "100%",
    height: "100%",
    display: "flex",
    justifyContent: "center",
    alignItems: "center",
  },
  content: {
    position: "unset",
    maxWidth: "360px",
    border: "none",
    borderRadius: "16px",
    padding: "32px",
    overflow: "auto",
    boxShadow: "0rem 2rem 2rem rgba(0, 0, 0, 0.2)",
  },
};

export default function Modal(
  props: ModalProps & { children: React.ReactNode },
) {
  return (
    <ReactModal
      style={ModalStyle}
      isOpen={props.isOpen}
      onRequestClose={props.close}
    >
      <div className="modal">{props.children}</div>
    </ReactModal>
  );
}
