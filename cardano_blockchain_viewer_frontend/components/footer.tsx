export function Footer() {
  return (
    <footer className="border-t border-border/20 bg-card/95 backdrop-blur-sm mt-16">
      <div className="container mx-auto px-4 py-16">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8 mb-8">
          {/* About Section */}
          <div className="space-y-4">
            <h3 className="text-xl font-bold text-foreground flex items-center gap-2">
              <span className="text-2xl">⛓️</span> Cardano Live Viewer
            </h3>
            <p className="text-sm text-muted-foreground leading-relaxed">
              Real-time Cardano PreProd testnet explorer with live data streaming, wallet authentication, and personalized transaction insights.
            </p>
            <div className="flex space-x-4">
              <a href="#" className="text-muted-foreground hover:text-foreground transition-colors">
                <span className="text-lg">🐦</span>
              </a>
              <a href="#" className="text-muted-foreground hover:text-foreground transition-colors">
                <span className="text-lg">💬</span>
              </a>
              <a href="#" className="text-muted-foreground hover:text-foreground transition-colors">
                <span className="text-lg">📘</span>
              </a>
              <a href="#" className="text-muted-foreground hover:text-foreground transition-colors">
                <span className="text-lg">🐙</span>
              </a>
            </div>
          </div>

          {/* Features Section */}
          <div className="space-y-4">
            <h4 className="text-lg font-semibold text-foreground">Features</h4>
            <ul className="space-y-2 text-sm text-muted-foreground">
              <li><a href="#" className="hover:text-foreground transition-colors">Live Dashboard</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">Block Explorer</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">Wallet Authentication</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">Transaction History</a></li>
            </ul>
          </div>

          {/* Resources Section */}
          <div className="space-y-4">
            <h4 className="text-lg font-semibold text-foreground">Resources</h4>
            <ul className="space-y-2 text-sm text-muted-foreground">
              <li><a href="#" className="hover:text-foreground transition-colors">Documentation</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">API Reference</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">GitHub Repository</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">Support</a></li>
            </ul>
          </div>

          {/* Legal & Contact Section */}
          <div className="space-y-4">
            <h4 className="text-lg font-semibold text-foreground">Legal & Contact</h4>
            <ul className="space-y-2 text-sm text-muted-foreground">
              <li><a href="#" className="hover:text-foreground transition-colors">Privacy Policy</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">Terms of Service</a></li>
              <li><a href="#" className="hover:text-foreground transition-colors">Cookie Policy</a></li>
              <li><a href="mailto:support@cardanoliveviewer.com" className="hover:text-foreground transition-colors">Contact Us</a></li>
            </ul>
          </div>
        </div>

        {/* Newsletter Section */}
        <div className="border-t border-border/20 pt-8 pb-6">
          <div className="max-w-md mx-auto text-center">
            <h4 className="text-lg font-semibold text-foreground mb-2">Stay Updated</h4>
            <p className="text-sm text-muted-foreground mb-4">
              Get the latest updates on Cardano blockchain developments and new features.
            </p>
            <div className="flex gap-2">
              <input
                type="email"
                placeholder="Enter your email"
                className="flex-1 px-3 py-2 bg-background border border-border rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-primary"
              />
              <button className="px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm font-medium hover:bg-primary/90 transition-colors">
                Subscribe
              </button>
            </div>
          </div>
        </div>

        {/* Bottom Section */}
        <div className="border-t border-border/20 pt-6 text-center">
          <p className="text-sm text-muted-foreground mb-2">
            Built with ❤️ for the Cardano community
          </p>
          <p className="text-sm text-muted-foreground">
            © {new Date().getFullYear()} Arpit-K-Sharma. All rights reserved.
          </p>
        </div>
      </div>
    </footer>
  )
}
