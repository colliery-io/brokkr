/**
 * Shared UI Components for Brokkr Control Panel
 *
 * This module contains reusable components used across all panels.
 */

import React, { useState, useEffect, useContext, createContext } from 'react';

// ==================== TOAST CONTEXT ====================

// Toast context for app-wide notifications
const ToastContext = createContext(null);

export const useToast = () => useContext(ToastContext);

// Toast notification component
const Toast = ({ message, type = 'info', onClose }) => (
  <div className={`toast toast-${type}`}>
    {message}
    <button onClick={onClose}>×</button>
  </div>
);

export const ToastProvider = ({ children }) => {
  const [toast, setToast] = useState(null);

  const showToast = (message, type = 'success') => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  };

  return (
    <ToastContext.Provider value={showToast}>
      {children}
      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </ToastContext.Provider>
  );
};

// ==================== UTILITIES ====================

// Helper to extract user-friendly error message
export const getErrorMessage = (error) => {
  if (error?.response?.data?.message) return error.response.data.message;
  if (error?.response?.data?.error) return error.response.data.error;
  if (error?.message) return error.message;
  return 'An unexpected error occurred';
};

// ==================== TAG ====================

export const Tag = ({ children, onRemove, variant = 'default' }) => (
  <span className={`tag tag-${variant}`}>
    {children}
    {onRemove && <button onClick={onRemove} className="tag-remove">×</button>}
  </span>
);

// ==================== SECTION ====================

export const Section = ({ title, icon, children, defaultOpen = false, count }) => {
  const [open, setOpen] = useState(defaultOpen);
  return (
    <div className={`section ${open ? 'open' : ''}`}>
      <button className="section-header" onClick={() => setOpen(!open)}>
        <span className="section-icon">{icon}</span>
        <span className="section-title">{title}</span>
        {count !== undefined && <span className="section-count">{count}</span>}
        <span className="section-arrow">{open ? '▼' : '▶'}</span>
      </button>
      {open && <div className="section-content">{children}</div>}
    </div>
  );
};

// ==================== INLINE ADD ====================

export const InlineAdd = ({ placeholder, onAdd, fields = 1 }) => {
  const [values, setValues] = useState(fields === 1 ? '' : { key: '', value: '' });
  const handleSubmit = (e) => {
    e.preventDefault();
    if (fields === 1 && values.trim()) {
      onAdd(values.trim());
      setValues('');
    } else if (fields === 2 && values.key.trim() && values.value.trim()) {
      onAdd(values.key.trim(), values.value.trim());
      setValues({ key: '', value: '' });
    }
  };
  return (
    <form onSubmit={handleSubmit} className="inline-add">
      {fields === 1 ? (
        <input value={values} onChange={(e) => setValues(e.target.value)} placeholder={placeholder} />
      ) : (
        <>
          <input value={values.key} onChange={(e) => setValues({ ...values, key: e.target.value })} placeholder="key" />
          <input value={values.value} onChange={(e) => setValues({ ...values, value: e.target.value })} placeholder="value" />
        </>
      )}
      <button type="submit">+</button>
    </form>
  );
};

// ==================== STATUS ====================

export const Status = ({ status }) => {
  const s = (status || '').toLowerCase();
  const color = s === 'active' || s === 'success' || s === 'healthy' ? 'green' :
                s === 'inactive' || s === 'failure' || s === 'failed' ? 'red' :
                s === 'pending' || s === 'in_progress' ? 'yellow' : 'gray';
  return <span className={`status status-${color}`}>{status}</span>;
};

// ==================== PAGINATION ====================

export const Pagination = ({ page, totalPages, onPageChange, pageSize, onPageSizeChange, total }) => (
  <div className="pagination">
    <div className="pagination-info">
      <span className="dim">Showing {Math.min((page - 1) * pageSize + 1, total)}-{Math.min(page * pageSize, total)} of {total}</span>
      <select value={pageSize} onChange={(e) => onPageSizeChange(parseInt(e.target.value))} className="pagination-size">
        <option value="10">10</option>
        <option value="25">25</option>
        <option value="50">50</option>
        <option value="100">100</option>
      </select>
    </div>
    <div className="pagination-controls">
      <button onClick={() => onPageChange(1)} disabled={page === 1} className="btn-icon">«</button>
      <button onClick={() => onPageChange(page - 1)} disabled={page === 1} className="btn-icon">‹</button>
      <span className="pagination-current">{page} / {totalPages || 1}</span>
      <button onClick={() => onPageChange(page + 1)} disabled={page >= totalPages} className="btn-icon">›</button>
      <button onClick={() => onPageChange(totalPages)} disabled={page >= totalPages} className="btn-icon">»</button>
    </div>
  </div>
);

// Hook for pagination state
export const usePagination = (items, defaultPageSize = 25) => {
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(defaultPageSize);

  const totalPages = Math.ceil(items.length / pageSize);
  const paginatedItems = items.slice((page - 1) * pageSize, page * pageSize);

  // Reset to page 1 when items or page size changes
  useEffect(() => {
    if (page > totalPages && totalPages > 0) setPage(totalPages);
    else if (page < 1) setPage(1);
  }, [items.length, pageSize, page, totalPages]);

  return {
    page,
    setPage,
    pageSize,
    setPageSize: (size) => { setPageSize(size); setPage(1); },
    totalPages,
    paginatedItems,
    total: items.length
  };
};

// ==================== MODAL ====================

export const Modal = ({ title, onClose, children }) => (
  <div className="modal-overlay" onClick={onClose}>
    <div className="modal" onClick={(e) => e.stopPropagation()}>
      <div className="modal-header">
        <h3>{title}</h3>
        <button onClick={onClose} className="modal-close">×</button>
      </div>
      <div className="modal-body">{children}</div>
    </div>
  </div>
);
