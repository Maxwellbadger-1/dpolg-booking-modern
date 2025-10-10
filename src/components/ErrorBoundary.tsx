import React, { Component, ErrorInfo, ReactNode } from 'react';
import { AlertTriangle, RefreshCw } from 'lucide-react';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

/**
 * Error Boundary Component - Fängt React-Fehler ab und verhindert White Screen
 *
 * Best Practice von professionellen Apps (Figma, Notion, VSCode)
 * - Zeigt User-freundliche Fehlermeldung
 * - Logged Error Details für Debugging
 * - Bietet "Neu laden" Button
 */
class ErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false,
    error: null,
    errorInfo: null,
  };

  public static getDerivedStateFromError(error: Error): State {
    console.error('🚨 ErrorBoundary caught error:', error);
    return { hasError: true, error, errorInfo: null };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('═══════════════════════════════════════');
    console.error('💥 ERROR BOUNDARY CAUGHT ERROR:');
    console.error('Error:', error);
    console.error('Error Message:', error.message);
    console.error('Stack Trace:', error.stack);
    console.error('Component Stack:', errorInfo.componentStack);
    console.error('═══════════════════════════════════════');

    this.setState({
      error,
      errorInfo,
    });
  }

  private handleReload = () => {
    // Reset error state and reload component
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
    });
  };

  public render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl p-8 max-w-2xl w-full border-2 border-red-500">
            {/* Header */}
            <div className="flex items-start gap-4 mb-6">
              <div className="p-3 bg-red-500/10 rounded-full flex-shrink-0">
                <AlertTriangle className="w-8 h-8 text-red-400" />
              </div>
              <div className="flex-1">
                <h2 className="text-2xl font-bold text-white mb-2">
                  Etwas ist schiefgegangen
                </h2>
                <p className="text-slate-300">
                  Ein unerwarteter Fehler ist aufgetreten. Bitte versuchen Sie, die Komponente neu zu laden.
                </p>
              </div>
            </div>

            {/* Error Details */}
            {this.state.error && (
              <div className="bg-slate-900 rounded-lg p-4 mb-6">
                <div className="text-sm text-slate-400 mb-2 font-semibold">
                  Fehlerdetails:
                </div>
                <div className="text-sm text-red-300 font-mono break-all">
                  {this.state.error.toString()}
                </div>
                {this.state.error.stack && (
                  <details className="mt-3">
                    <summary className="text-sm text-slate-400 cursor-pointer hover:text-slate-300">
                      Stack Trace anzeigen
                    </summary>
                    <pre className="mt-2 text-xs text-slate-400 overflow-auto max-h-40">
                      {this.state.error.stack}
                    </pre>
                  </details>
                )}
              </div>
            )}

            {/* Actions */}
            <div className="flex gap-3">
              <button
                onClick={this.handleReload}
                className="flex-1 px-4 py-3 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-lg transition-colors flex items-center justify-center gap-2"
              >
                <RefreshCw className="w-5 h-5" />
                Komponente neu laden
              </button>
              <button
                onClick={() => window.location.reload()}
                className="flex-1 px-4 py-3 bg-slate-700 hover:bg-slate-600 text-white font-semibold rounded-lg transition-colors"
              >
                Seite neu laden
              </button>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;
