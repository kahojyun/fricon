---
title: Defer Non-Critical Third-Party Libraries
impact: MEDIUM
impactDescription: loads after initial render
tags: bundle, third-party, analytics, defer
---

## Defer Non-Critical Third-Party Libraries

Analytics, logging, and error tracking don't block user interaction. Load them after initial render.

**Incorrect (blocks initial bundle):**

```tsx
import { Analytics } from '@vercel/analytics/react'

export default function RootLayout({ children }) {
  return (
    <html>
      <body>
        {children}
        <Analytics />
      </body>
    </html>
  )
}
```

**Correct (loads after initial render):**

```tsx
import { useEffect, useState } from 'react'

let cachedAnalytics: React.ComponentType | null = null

export default function RootLayout({ children }) {
  const [Analytics, setAnalytics] = useState<React.ComponentType | null>(cachedAnalytics)

  useEffect(() => {
    if (cachedAnalytics) return
    void import('@vercel/analytics/react').then(m => {
      cachedAnalytics = m.Analytics
      setAnalytics(() => m.Analytics)
    })
  }, [])

  return (
    <>
      {children}
      {Analytics ? <Analytics /> : null}
    </>
  )
}
```
