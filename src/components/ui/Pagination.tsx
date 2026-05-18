import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from "lucide-react";
import { useTranslation } from "react-i18next";

interface PaginationProps {
  page: number;
  totalPages: number;
  total: number;
  pageSize: number;
  isLoading?: boolean;
  onPageChange: (page: number) => void;
}

export function Pagination({
  page,
  totalPages,
  total,
  pageSize,
  isLoading = false,
  onPageChange,
}: PaginationProps) {
  const { t } = useTranslation("common");

  if (totalPages <= 1) {
    return null;
  }

  const startItem = (page - 1) * pageSize + 1;
  const endItem = Math.min(page * pageSize, total);

  const getPageNumbers = () => {
    const pages: (number | "...")[] = [];
    const maxVisible = 7;

    if (totalPages <= maxVisible) {
      for (let i = 1; i <= totalPages; i++) {
        pages.push(i);
      }
    } else {
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
        {t("pagination.info", { start: startItem, end: endItem, total })}
      </div>

      <div className="pagination-controls">
        <button
          className="pagination-btn"
          onClick={() => onPageChange(1)}
          disabled={page === 1 || isLoading}
          title={t("pagination.first")}
        >
          <ChevronsLeft size={14} />
        </button>

        <button
          className="pagination-btn"
          onClick={() => onPageChange(page - 1)}
          disabled={page === 1 || isLoading}
          title={t("pagination.prev")}
        >
          <ChevronLeft size={14} />
        </button>

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

        <button
          className="pagination-btn"
          onClick={() => onPageChange(page + 1)}
          disabled={page === totalPages || isLoading}
          title={t("pagination.next")}
        >
          <ChevronRight size={14} />
        </button>

        <button
          className="pagination-btn"
          onClick={() => onPageChange(totalPages)}
          disabled={page === totalPages || isLoading}
          title={t("pagination.last")}
        >
          <ChevronsRight size={14} />
        </button>
      </div>

      <div className="pagination-jump">
        <span>{t("pagination.jumpTo")}</span>
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
        <span>{t("pagination.page")}</span>
      </div>
    </div>
  );
}
