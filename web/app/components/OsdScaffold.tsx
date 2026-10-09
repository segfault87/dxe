import { Outlet } from "react-router";

import "./OsdScaffold.css";

export default function OsdScaffold() {
  return (
    <main>
      <div className="osd-contents">
        <Outlet />
      </div>
    </main>
  );
}
