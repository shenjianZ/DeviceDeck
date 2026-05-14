import { AlertTriangle, Check, FileText, Play, RefreshCw, Zap } from "lucide-react";
import type { Page } from "../../types";

interface DashboardPageProps {
  onlineCount: number;
  runningCount: number;
  totalSessions: number;
  onNav: (page: Page) => void;
}

export function DashboardPage({
  onlineCount,
  runningCount,
  totalSessions,
  onNav,
}: DashboardPageProps) {
  return (
    <div className="col" style={{ gap: 14 }}>
      <div className="sec-title flush">环境状态</div>
      <div className="grid3">
        <div className="env-row">
          <div className="env-icon" style={{ background: "var(--ok-s)", color: "var(--ok)" }}>
            A
          </div>
          <div className="col">
            <span style={{ fontWeight: 600, fontSize: 13 }}>ADB</span>
            <span style={{ fontSize: 11, color: "var(--t2)" }}>v34.0.5 · 可用</span>
          </div>
        </div>
        <div className="env-row">
          <div className="env-icon" style={{ background: "var(--ok-s)", color: "var(--ok)" }}>
            S
          </div>
          <div className="col">
            <span style={{ fontWeight: 600, fontSize: 13 }}>Scrcpy</span>
            <span style={{ fontSize: 11, color: "var(--t2)" }}>v2.4 · 可用</span>
          </div>
        </div>
        <div className="env-row">
          <div className="env-icon" style={{ background: "var(--acc-s)", color: "var(--acc)" }}>
            D
          </div>
          <div className="col">
            <span style={{ fontWeight: 600, fontSize: 13 }}>Android Provider</span>
            <span style={{ fontSize: 11, color: "var(--t2)" }}>运行中</span>
          </div>
        </div>
      </div>

      <div className="sec-title">概览</div>
      <div className="grid3">
        <div className="card">
          <div className="card-title">已连接设备</div>
          <div className="card-val">{onlineCount}</div>
        </div>
        <div className="card">
          <div className="card-title">运行中投屏</div>
          <div className="card-val" style={{ color: runningCount ? "var(--ok)" : "var(--t2)" }}>
            {runningCount}
          </div>
        </div>
        <div className="card">
          <div className="card-title">总会话数</div>
          <div className="card-val">{totalSessions}</div>
        </div>
      </div>

      <div className="sec-title">快速操作</div>
      <div className="row">
        <button className="btn btn-p" onClick={() => onNav("devices")} type="button">
          <RefreshCw size={14} />
          扫描设备
        </button>
        <button className="btn btn-s" onClick={() => onNav("mirror")} type="button">
          <Play size={14} />
          开始投屏
        </button>
        <button className="btn btn-s" onClick={() => onNav("logs")} type="button">
          <FileText size={14} />
          查看日志
        </button>
      </div>

      <div className="sec-title">最近活动</div>
      <div className="card" style={{ display: "flex", flexDirection: "column", gap: 6, fontSize: 12 }}>
        <div className="row" style={{ color: "var(--t1)" }}>
          <Play size={13} />
          Pixel 8 Pro 投屏已启动
          <span className="mono" style={{ color: "var(--t2)", marginLeft: "auto" }}>3 分钟前</span>
        </div>
        <div className="row" style={{ color: "var(--t1)" }}>
          <Check size={13} />
          ADB 环境检测通过
          <span className="mono" style={{ color: "var(--t2)", marginLeft: "auto" }}>5 分钟前</span>
        </div>
        <div className="row" style={{ color: "var(--err)" }}>
          <AlertTriangle size={13} />
          Xiaomi 14 未授权
          <span className="mono" style={{ color: "var(--t2)", marginLeft: "auto" }}>5 分钟前</span>
        </div>
        <div className="row" style={{ color: "var(--t1)" }}>
          <Zap size={13} />
          DeviceDeck 启动完成
          <span className="mono" style={{ color: "var(--t2)", marginLeft: "auto" }}>5 分钟前</span>
        </div>
      </div>
    </div>
  );
}
