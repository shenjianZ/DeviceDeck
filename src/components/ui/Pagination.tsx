import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from "lucide-react";

interface PaginationProps {
  /** 当前页码 */
  page: number;
  /** 总页数 */
  totalPages: number;
  /** 总记录数 */
  total: number;
  /** 每页大小 */
  pageSize: number;
  /** 是否正在加载 */
  isLoading?: boolean;
  /** 页码变化回调 */
  onPageChange: (page: number) => void;
}

/**
 * 分页组件
 * 提供首页、上一页、页码显示、下一页、末页的导航功能
 */
export function Pagination({
  page,
  totalPages,
  total,
  pageSize,
  isLoading = false,
  onPageChange,
}: PaginationProps) {
  if (totalPages <= 1) {
    return null;
  }

  const startItem = (page - 1) * pageSize + 1;
  const endItem = Math.min(page * pageSize, total);

  // 生成页码按钮
  const getPageNumbers = () => {
    const pages: (number | "...")[] = [];
    const maxVisible = 7;

    if (totalPages <= maxVisible) {
      // 总页数少，全部显示
      for (let i = 1; i <= totalPages; i++) {
        pages.push(i);
      }
    } else {
      // 总页数多，显示部分
      pages.push(1);

      if (page > 3) {
        pages.push("...");
      }

      const start = Math.max(2, page - 1);
      const end = Math.min(totalPages - 1, page + 1);

      for (let i = start; i <= end; i++) {
        pages.push(i);
      }

      if (page < totalPages - 2) {
        pages.push("...");
      }

      pages.push(totalPages);
    }

    return pages;
  };

  return (
    <div className="pagination">
      <div className="pagination-info">
        显示 {startItem}-{endItem} 条，共 {total} 条
      </div>

      <div className="pagination-controls">
        {/* 首页 */}
        <button
          className="pagination-btn"
          onClick={() => onPageChange(1)}
          disabled={page === 1 || isLoading}
          title="首页"
        >
          <ChevronsLeft size={14} />
        </button>

        {/* 上一页 */}
        <button
          className="pagination-btn"
          onClick={() => onPageChange(page - 1)}
          disabled={page === 1 || isLoading}
          title="上一页"
        >
          <ChevronLeft size={14} />
        </button>

        {/* 页码 */}
        {getPageNumbers().map((pageNum, index) =>
          pageNum === "..." ? (
            <span key={`ellipsis-${index}`} className="pagination-ellipsis">
              ...
            </span>
          ) : (
            <button
              key={pageNum}
              className={`pagination-btn pagination-page ${
                pageNum === page ? "active" : ""
              }`}
              onClick={() => onPageChange(pageNum as number)}
              disabled={isLoading}
            >
              {pageNum}
            </button>
          )
        )}

        {/* 下一页 */}
        <button
          className="pagination-btn"
          onClick={() => onPageChange(page + 1)}
          disabled={page === totalPages || isLoading}
          title="下一页"
        >
          <ChevronRight size={14} />
        </button>

        {/* 末页 */}
        <button
          className="pagination-btn"
          onClick={() => onPageChange(totalPages)}
          disabled={page === totalPages || isLoading}
          title="末页"
        >
          <ChevronsRight size={14} />
        </button>
      </div>

      {/* 快速跳转 */}
      <div className="pagination-jump">
        <span>跳转到</span>
        <input
          type="number"
          min={1}
          max={totalPages}
          value={page}
          onChange={(e) => {
            const targetPage = parseInt(e.target.value, 10);
            if (targetPage >= 1 && targetPage <= totalPages) {
              onPageChange(targetPage);
            }
          }}
          className="pagination-jump-input"
          disabled={isLoading}
        />
        <span>页</span>
      </div>
    </div>
  );
}
