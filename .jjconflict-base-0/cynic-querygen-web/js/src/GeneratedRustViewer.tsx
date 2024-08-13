import * as React from "react";
import { useRef, useEffect, useState } from "react";
import CodeMirror from "codemirror";
import "codemirror/mode/rust/rust";

interface GeneratedRustViewer {
  generatedCode: string;
}

const GeneratedRustViewer: React.FC<GeneratedRustViewer> = ({
  generatedCode,
}) => {
  const [viewerOpen, setOpen] = useState(true);
  const [viewerHeight, setHeight] = useState(512);
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
          paddingLeft: 8,
        }}
        onMouseDown={(downEvent) =>
          handleEditorResizeStart({
            downEvent,
            wasOpen: viewerOpen,
            hadHeight: viewerHeight,
            setOpen,
            setHeight,
          })
        }
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
      <Viewer value={generatedCode} />
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

  useEffect(() => {
    if (codeMirrorRef.current) {
      codeMirrorRef.current.setValue(props.value);
    }
  }, [codeMirrorRef.current, props.value]);

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

interface HandleEditorArg {
  downEvent: React.MouseEvent<HTMLDivElement>;
  wasOpen: boolean;
  hadHeight: number;
  setOpen: (bool) => void;
  setHeight: (number) => void;
}

const handleEditorResizeStart = ({
  downEvent,
  wasOpen,
  hadHeight,
  setOpen,
  setHeight,
}: HandleEditorArg) => {
  downEvent.preventDefault();

  // Forgive me for this crime
  const editorBar =
    downEvent.currentTarget.parentElement.parentElement.parentElement
      .parentElement;

  let didMove = false;
  const offset = downEvent.clientY - getTop(downEvent.target as HTMLElement);

  let onMouseMove = (moveEvent: MouseEvent) => {
    if (moveEvent.buttons === 0) {
      return onMouseUp!();
    }

    didMove = true;

    const topSize = moveEvent.clientY - getTop(editorBar) - offset;
    const bottomSize = editorBar.clientHeight - topSize;

    if (bottomSize < 60) {
      setOpen(false);
      setHeight(hadHeight);
    } else {
      setOpen(true);
      setHeight(bottomSize);
    }
  };

  let onMouseUp = () => {
    if (!didMove) {
      setOpen(!wasOpen);
    }

    document.removeEventListener("mousemove", onMouseMove!);
    document.removeEventListener("mouseup", onMouseUp!);
    onMouseMove = null;
    onMouseUp = null;
  };

  document.addEventListener("mousemove", onMouseMove);
  document.addEventListener("mouseup", onMouseUp);
};

export function getTop(initialElem: HTMLElement) {
  let pt = 0;
  let elem = initialElem;
  while (elem.offsetParent) {
    pt += elem.offsetTop;
    elem = elem.offsetParent as HTMLElement;
  }
  return pt;
}
