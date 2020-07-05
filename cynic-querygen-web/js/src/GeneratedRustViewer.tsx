import * as React from "react";
import { useRef, useEffect, useState } from "react";
import CodeMirror from "codemirror";

const GeneratedRustViewer: React.FC = () => {
  const [viewerOpen, setOpen] = useState(true);
  const [viewerHeight, setHeight] = useState(256);
  const viewerStyle = {
    height: viewerOpen ? viewerHeight : undefined,
  };

  const variableEditorActive = true;

  return (
    <section
      className="variable-editor secondary-editor"
      style={viewerStyle}
      aria-label="Generated Rust"
    >
      <div
        className="secondary-editor-title variable-editor-title"
        id="secondary-editor-title"
        style={{
          cursor: viewerOpen ? "row-resize" : "n-resize",
        }}
      >
        <div
          style={{
            cursor: "pointer",
            color: variableEditorActive ? "#000" : "gray",
            display: "inline-block",
          }}
          onClick={() => setOpen(!viewerOpen)}
        >
          {"Generated Rust"}
        </div>
      </div>
      <Viewer />
    </section>
  );
};

export default GeneratedRustViewer;

interface ViewerProps {
  value?: string;
  editorTheme?: string;
}

const Viewer: React.FC<ViewerProps> = (props) => {
  const sectionRef = useRef<HTMLElement | null>();
  const codeMirrorRef = useRef<CodeMirror | null>();

  useEffect(() => {
    if (sectionRef.current) {
      codeMirrorRef.current = CodeMirror(sectionRef.current, {
        lineWrapping: true,
        value: props.value || "",
        readOnly: true,
        theme: props.editorTheme || "graphiql",
        mode: "rust",
        keyMap: "sublime",
      });
    }

    () => {
      codeMirrorRef.current = null;
    };
  }, [sectionRef.current]);

  return (
    <section
      className="result-window"
      aria-label="Generated Rust"
      aria-live="polite"
      aria-atomic="true"
      ref={sectionRef}
    />
  );
};

