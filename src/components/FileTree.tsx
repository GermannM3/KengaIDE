import type { ProjectTreeNode } from "../types";

interface FileTreeListProps {
  nodes: ProjectTreeNode[];
  depth?: number;
  expanded: Set<string>;
  onToggle: (path: string) => void;
  onFileClick: (path: string, e?: React.MouseEvent) => void;
  currentFilePath: string | null;
}

export function FileTreeList({
  nodes,
  depth = 0,
  expanded,
  onToggle,
  onFileClick,
  currentFilePath,
}: FileTreeListProps) {
  return (
    <>
      {nodes.map((node) => {
        const isDir = node.kind === "dir";
        const hasChildren = isDir && node.children && node.children.length > 0;
        const isExpanded = hasChildren && expanded.has(node.path);
        const isCurrent = !isDir && node.path === currentFilePath;
        return (
          <div key={node.path} style={{ marginLeft: depth * 12 }}>
            {isDir ? (
              <button
                type="button"
                className="kenga-file-tree-item"
                onClick={() => onToggle(node.path)}
              >
                <span style={{ width: 14, display: "inline-block" }}>
                  {isExpanded ? "▼" : "▶"}
                </span>
                <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                  {node.name}
                </span>
              </button>
            ) : (
              <button
                type="button"
                className={`kenga-file-tree-file ${isCurrent ? "kenga-active" : ""}`}
                onClick={(e) => onFileClick(node.path, e)}
                title={node.path}
              >
                {node.name}
              </button>
            )}
            {isDir && hasChildren && isExpanded && (
              <FileTreeList
                nodes={node.children!}
                depth={depth + 1}
                expanded={expanded}
                onToggle={onToggle}
                onFileClick={onFileClick}
                currentFilePath={currentFilePath}
              />
            )}
          </div>
        );
      })}
    </>
  );
}
