import React from "react";

import "./Section.css";

export default function Section({
  id,
  title,
  children,
}: {
  id: string;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="section" id={id}>
      <div className="title">
        <a href={`#${id}`}> {title} </a>
      </div>
      <div className="body">{children}</div>
    </div>
  );
}
