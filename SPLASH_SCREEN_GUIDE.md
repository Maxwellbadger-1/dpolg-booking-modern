# 🎬 Splash Screen / Ladebildschirm Guide
## DPolG Buchungssystem - Professionelle Ladeanimationen

**Erstellt:** 2025-10-18
**Status:** Für zukünftige Implementierung

---

## 📋 Inhaltsverzeichnis

1. [Übersicht](#übersicht)
2. [Option 1: Lottie Animationen (EMPFOHLEN)](#option-1-lottie-animationen-empfohlen)
3. [Option 2: SVG + CSS Animationen](#option-2-svg--css-animationen)
4. [Option 3: Tauri Native Splash Screen](#option-3-tauri-native-splash-screen)
5. [Vergleichstabelle](#vergleichstabelle)
6. [Implementierungsschritte](#implementierungsschritte)
7. [Beispiel-Code](#beispiel-code)
8. [Tools & Ressourcen](#tools--ressourcen)

---

## 🎯 Übersicht

Professionelle Desktop-Apps zeigen beim Start einen Ladebildschirm (Splash Screen) mit:
- Firmenlogo + Animation
- Firmennamen
- Ladefortschritt/Indikator
- Branding-Farben

**Beispiele:**
- **Slack:** Lottie Animation mit Logo-Morphing
- **Notion:** Minimalistisches SVG mit Fade-in
- **VSCode:** Statisches PNG mit Progress Bar
- **Linear:** Lottie mit komplexer Logo-Animation

---

## ⭐ Option 1: Lottie Animationen (EMPFOHLEN)

### Was ist Lottie?

Lottie ist ein JSON-basiertes Animationsformat, entwickelt von Airbnb:
- ✅ Sehr kleine Dateigröße (~10-50 KB)
- ✅ Perfekt skalierbar (Vektor-basiert)
- ✅ Butterweiche 60 FPS Animationen
- ✅ Kann von Designern ohne Code erstellt werden
- ✅ Professionellster Look

### Workflow: PNG Logo → Lottie Animation

#### Schritt 1: Recraft.ai (KOSTENLOS & EINFACH!)

**URL:** https://www.recraft.ai/

**Anleitung:**
1. Gehe zu Recraft.ai
2. Upload dein PNG Logo
3. AI konvertiert es automatisch zu Vektor
4. Klicke "Export" → "Lottie"
5. Download `.json` Datei
6. Fertig! (dauert ~2 Minuten)

**Vorteile:**
- Komplett kostenlos
- Keine Anmeldung nötig
- AI-powered Vektorisierung
- Direkt Lottie Export

#### Schritt 2: LottieFiles AI Tools (PROFESSIONELL)

**URL:** https://lottiefiles.com/ai

**Features:**
- **Raster to Vector:** Konvertiert PNG/JPG zu Vektor
- **Motion Copilot:** AI erstellt Animationen aus Text-Beschreibungen
- **AI Prompt to Vector:** Logo aus Text-Prompt generieren

**Anleitung:**
1. Upload PNG in LottieFiles Figma Plugin
2. "Raster to Vector" nutzen
3. Mit "Motion Copilot" Animationen hinzufügen (z.B. "Make the logo pulse and fade in")
4. Als Lottie exportieren

**Vorteil:**
- Professionellste Lösung
- Mehr Kontrolle über Animationen
- Community mit 1000+ fertigen Animationen

#### Schritt 3: Fertige Lottie Animationen anpassen

**URL:** https://lottiefiles.com/free-animations

**Anleitung:**
1. Suche nach "logo animation", "loading", "corporate"
2. Download kostenlose Lottie Datei
3. Im **Lottie Creator** öffnen
4. Farben/Text auf dein Branding anpassen
5. Export

**Beispiel-Suchen:**
- "corporate logo animation"
- "loading spinner"
- "brand reveal"
- "professional intro"

### Dateien die du brauchst

```
assets/
  └── splash/
      ├── logo-animation.json     # Lottie Animation (von Recraft.ai)
      └── logo-static.png          # Fallback für alte Systeme
```

### Integration in Tauri

**1. Installiere Lottie Library:**

```bash
npm install lottie-react
```

**2. Erstelle Splash Screen Component:**

```tsx
// src/components/SplashScreen.tsx
import Lottie from 'lottie-react';
import logoAnimation from '../assets/splash/logo-animation.json';

export function SplashScreen() {
  return (
    <div className="fixed inset-0 bg-gradient-to-br from-blue-900 to-slate-900 flex items-center justify-center">
      {/* Logo Animation */}
      <div className="w-64 h-64">
        <Lottie
          animationData={logoAnimation}
          loop={true}
          autoplay={true}
        />
      </div>

      {/* Firmenname */}
      <p className="absolute bottom-20 text-white text-lg font-semibold">
        DPolG Buchungssystem
      </p>

      {/* Ladeindikator */}
      <div className="absolute bottom-10 flex items-center gap-2">
        <div className="w-2 h-2 bg-blue-400 rounded-full animate-pulse"></div>
        <p className="text-slate-300 text-sm">Wird geladen...</p>
      </div>
    </div>
  );
}
```

**3. In App.tsx einbinden:**

```tsx
import { useState, useEffect } from 'react';
import SplashScreen from './components/SplashScreen';

function App() {
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    // Simuliere Ladezeit (3 Sekunden)
    setTimeout(() => {
      setIsLoading(false);
    }, 3000);
  }, []);

  if (isLoading) {
    return <SplashScreen />;
  }

  return (
    <div>
      {/* Deine normale App */}
    </div>
  );
}
```

---

## 🎨 Option 2: SVG + CSS Animationen

Einfachere Alternative ohne externe Dependencies.

### Dateien die du brauchst

```
assets/
  └── splash/
      └── logo.svg    # Animiertes SVG
```

### Beispiel animiertes SVG

```svg
<!-- assets/splash/logo.svg -->
<svg width="200" height="200" viewBox="0 0 200 200">
  <style>
    @keyframes rotate {
      from { transform: rotate(0deg); }
      to { transform: rotate(360deg); }
    }
    @keyframes fadeIn {
      from { opacity: 0; transform: scale(0.8); }
      to { opacity: 1; transform: scale(1); }
    }
    @keyframes pulse {
      0%, 100% { transform: scale(1); }
      50% { transform: scale(1.05); }
    }

    .logo-circle {
      transform-origin: center;
      animation: rotate 2s ease-in-out infinite;
    }
    .logo-text {
      animation: fadeIn 1s ease-in;
    }
    .logo-container {
      animation: pulse 2s ease-in-out infinite;
    }
  </style>

  <g class="logo-container">
    <circle class="logo-circle" cx="100" cy="100" r="50" fill="#3B82F6" stroke="#60A5FA" stroke-width="3" />
    <text class="logo-text" x="100" y="110" text-anchor="middle" fill="white" font-size="20" font-weight="bold">
      DPolG
    </text>
  </g>
</svg>
```

### Integration

```tsx
// src/components/SplashScreen.tsx
export function SplashScreen() {
  return (
    <div className="fixed inset-0 bg-gradient-to-br from-blue-900 to-slate-900 flex items-center justify-center">
      <div className="text-center">
        <img src="/assets/splash/logo.svg" alt="Logo" className="w-64 h-64 mx-auto" />
        <h1 className="text-white text-2xl font-bold mt-4">DPolG Buchungssystem</h1>
        <div className="mt-6 flex items-center justify-center gap-2">
          <div className="w-2 h-2 bg-blue-400 rounded-full animate-pulse"></div>
          <p className="text-slate-300">Wird geladen...</p>
        </div>
      </div>
    </div>
  );
}
```

**Vorteile:**
- ✅ Keine Dependencies
- ✅ Sehr kleine Dateigröße (~5 KB)
- ✅ Einfach anpassbar
- ✅ Funktioniert überall

---

## 🔧 Option 3: Tauri Native Splash Screen

### Dateien die du brauchst

```
src-tauri/icons/
  └── splash.png    # 1024x1024 PNG mit Transparenz
```

### Konfiguration in `tauri.conf.json`

```json
{
  "tauri": {
    "windows": [{
      "title": "DPolG Buchungssystem",
      "width": 1400,
      "height": 900,
      "decorations": false,
      "transparent": true,
      "fullscreen": false,
      "resizable": true
    }]
  },
  "plugins": {
    "window": {
      "windows": [{
        "label": "splash",
        "url": "/splash.html",
        "width": 600,
        "height": 400,
        "center": true,
        "resizable": false,
        "title": "Loading...",
        "decorations": false,
        "transparent": true
      }]
    }
  }
}
```

### Erstelle `splash.html`

```html
<!DOCTYPE html>
<html>
<head>
  <style>
    body {
      margin: 0;
      background: linear-gradient(135deg, #1e3a8a, #1e293b);
      display: flex;
      align-items: center;
      justify-content: center;
      height: 100vh;
      overflow: hidden;
    }
    .splash-container {
      text-align: center;
      animation: fadeIn 0.5s ease-in;
    }
    @keyframes fadeIn {
      from { opacity: 0; transform: scale(0.95); }
      to { opacity: 1; transform: scale(1); }
    }
    .logo {
      width: 200px;
      height: 200px;
      margin-bottom: 20px;
      animation: pulse 2s ease-in-out infinite;
    }
    @keyframes pulse {
      0%, 100% { transform: scale(1); }
      50% { transform: scale(1.05); }
    }
    .spinner {
      width: 40px;
      height: 40px;
      margin: 20px auto;
      border: 4px solid rgba(255, 255, 255, 0.3);
      border-top-color: white;
      border-radius: 50%;
      animation: spin 1s linear infinite;
    }
    @keyframes spin {
      to { transform: rotate(360deg); }
    }
  </style>
</head>
<body>
  <div class="splash-container">
    <img src="/icons/splash.png" class="logo" alt="Logo">
    <h1 style="color: white; font-family: sans-serif; margin: 0;">DPolG Buchungssystem</h1>
    <div class="spinner"></div>
  </div>
</body>
</html>
```

**Vorteile:**
- ✅ Sehr einfach
- ✅ Native Tauri Integration
- ✅ Kein JavaScript nötig

---

## 📊 Vergleichstabelle

| Option | Komplexität | Professionalität | Dateigröße | Animationen | Dependencies |
|--------|-------------|------------------|------------|-------------|--------------|
| **Lottie** | Mittel | ⭐⭐⭐⭐⭐ | ~20 KB | Komplex möglich | lottie-react |
| **SVG + CSS** | Niedrig | ⭐⭐⭐⭐ | ~5 KB | Einfache Animationen | Keine |
| **Tauri Native** | Sehr niedrig | ⭐⭐⭐ | ~100 KB | Nur fade-in | Keine |
| **GIF** | Niedrig | ⭐⭐ | ~500 KB+ | ❌ Veraltet | Keine |

---

## 🛠️ Implementierungsschritte

### Phase 1: Vorbereitung (JETZT)

1. **Entscheide dich für eine Option:**
   - Lottie = Professionellster Look
   - SVG = Gute Balance zwischen Einfachheit und Qualität
   - Tauri Native = Schnellste Implementation

2. **Logo vorbereiten:**
   - Idealerweise PNG mit Transparenz
   - Mindestens 512x512 Pixel
   - Auf weißem/dunklem Hintergrund gut sichtbar

### Phase 2: Erstellung (Später)

#### Für Lottie:
1. Gehe zu **https://www.recraft.ai/**
2. Upload PNG Logo
3. Export als Lottie JSON
4. Speichere als `assets/splash/logo-animation.json`

#### Für SVG:
1. Logo in SVG konvertieren (mit Recraft.ai oder Illustrator)
2. Animationen in SVG einbauen (siehe Beispiel oben)
3. Speichere als `assets/splash/logo.svg`

### Phase 3: Integration

1. **Installiere Dependencies** (nur für Lottie):
   ```bash
   npm install lottie-react
   ```

2. **Erstelle Component:**
   ```bash
   # Datei erstellen
   touch src/components/SplashScreen.tsx
   ```

3. **Code einfügen** (siehe Beispiel-Code oben)

4. **In App.tsx integrieren:**
   - Splash Screen beim Start zeigen
   - Nach 2-3 Sekunden ausblenden
   - Zur Hauptapp überleiten

### Phase 4: Testing

1. **App neu kompilieren:**
   ```bash
   npm run tauri dev
   ```

2. **Testen:**
   - Splash Screen erscheint beim Start?
   - Animation läuft smooth?
   - Übergang zur App funktioniert?

3. **Fine-tuning:**
   - Ladezeit anpassen
   - Farben an Branding anpassen
   - Animation-Geschwindigkeit optimieren

---

## 💻 Beispiel-Code

### Vollständiger Splash Screen mit Lottie

```tsx
// src/components/SplashScreen.tsx
import { useEffect, useState } from 'react';
import Lottie from 'lottie-react';
import logoAnimation from '../assets/splash/logo-animation.json';

interface SplashScreenProps {
  onComplete?: () => void;
  minDuration?: number; // Minimum Zeit in ms
}

export function SplashScreen({ onComplete, minDuration = 2000 }: SplashScreenProps) {
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    const startTime = Date.now();

    // Simuliere Ladefortschritt
    const interval = setInterval(() => {
      setProgress(prev => {
        if (prev >= 100) {
          clearInterval(interval);

          // Warte mindestens minDuration
          const elapsed = Date.now() - startTime;
          const remaining = Math.max(0, minDuration - elapsed);

          setTimeout(() => {
            onComplete?.();
          }, remaining);

          return 100;
        }
        return prev + 10;
      });
    }, 200);

    return () => clearInterval(interval);
  }, [onComplete, minDuration]);

  return (
    <div className="fixed inset-0 bg-gradient-to-br from-blue-900 via-slate-900 to-slate-800 flex items-center justify-center">
      {/* Logo Animation */}
      <div className="relative">
        <div className="w-64 h-64">
          <Lottie
            animationData={logoAnimation}
            loop={true}
            autoplay={true}
          />
        </div>

        {/* Firmenname */}
        <div className="absolute -bottom-16 left-1/2 transform -translate-x-1/2 whitespace-nowrap">
          <h1 className="text-white text-2xl font-bold tracking-wide">
            DPolG Buchungssystem
          </h1>
        </div>
      </div>

      {/* Progress Bar */}
      <div className="absolute bottom-20 left-1/2 transform -translate-x-1/2 w-64">
        <div className="h-1 bg-slate-700 rounded-full overflow-hidden">
          <div
            className="h-full bg-gradient-to-r from-blue-400 to-blue-600 transition-all duration-300"
            style={{ width: `${progress}%` }}
          />
        </div>
        <p className="text-slate-400 text-sm text-center mt-2">
          {progress < 100 ? 'Wird geladen...' : 'Bereit!'}
        </p>
      </div>

      {/* Version Info */}
      <div className="absolute bottom-4 right-4 text-slate-500 text-xs">
        v1.6.0
      </div>
    </div>
  );
}
```

### Integration in App.tsx

```tsx
// src/App.tsx
import { useState } from 'react';
import SplashScreen from './components/SplashScreen';
import MainApp from './components/MainApp';

function App() {
  const [showSplash, setShowSplash] = useState(true);

  if (showSplash) {
    return <SplashScreen onComplete={() => setShowSplash(false)} minDuration={2000} />;
  }

  return <MainApp />;
}

export default App;
```

---

## 🔗 Tools & Ressourcen

### PNG → Lottie Konverter

1. **Recraft.ai** (EMPFOHLEN)
   - URL: https://www.recraft.ai/
   - Kostenlos, keine Anmeldung
   - AI-powered Vektorisierung
   - Direkt Lottie Export

2. **LottieFiles**
   - URL: https://lottiefiles.com/ai
   - AI Tools: Raster to Vector, Motion Copilot
   - Community mit 1000+ Animationen
   - Professionelle Features

3. **Lottie Creator**
   - URL: https://lottiefiles.com/lottie-creator
   - Web-basierter Editor
   - Motion Copilot AI
   - Interaktive Animationen

### Inspiration & fertige Animationen

1. **LottieFiles Free Animations**
   - URL: https://lottiefiles.com/free-animations
   - Tausende kostenlose Animationen
   - Filter: "logo", "loading", "corporate"

2. **IconScout Lottie**
   - URL: https://iconscout.com/lotties
   - Professionelle Animationen
   - Viele kostenlose Downloads

### Design Tools

1. **Adobe After Effects + Bodymovin**
   - Professionellste Lösung
   - Volle Kontrolle über Animationen
   - Export als Lottie JSON

2. **Figma mit Lottie Plugin**
   - Einfacher als After Effects
   - Gute Integration
   - Für einfache Animationen

---

## 📝 Checkliste

### Vor der Implementation

- [ ] Option gewählt (Lottie / SVG / Tauri Native)
- [ ] Logo als PNG vorbereitet (min. 512x512px)
- [ ] Branding-Farben definiert
- [ ] Gewünschte Animation-Art festgelegt

### Lottie Workflow

- [ ] Recraft.ai Account erstellt (optional)
- [ ] PNG zu Lottie konvertiert
- [ ] JSON Datei heruntergeladen
- [ ] In `assets/splash/` gespeichert
- [ ] `lottie-react` installiert

### Integration

- [ ] `SplashScreen.tsx` erstellt
- [ ] Code implementiert
- [ ] In `App.tsx` eingebunden
- [ ] Getestet im Dev Mode
- [ ] Ladezeit optimiert

### Final Testing

- [ ] Splash Screen erscheint beim Start
- [ ] Animation läuft smooth (60 FPS)
- [ ] Übergang zur App funktioniert
- [ ] Branding-Farben korrekt
- [ ] Performance gut (keine Verzögerung)

---

## 🎯 Empfehlung für DPolG Buchungssystem

### Kurzfristig (MVP):
**SVG + CSS Animation**
- Schnell implementiert
- Sieht professionell aus
- Keine externen Dependencies
- Perfekt für erste Version

### Langfristig (Polish):
**Lottie Animation**
- Upgrade wenn Designer verfügbar
- Für den "Wow"-Effekt
- Wenn Zeit für Feinschliff da ist

---

## 📞 Nächste Schritte

1. **Logo vorbereiten** (PNG, min. 512x512px)
2. **Option wählen** (Lottie empfohlen)
3. **Zu Recraft.ai gehen** und konvertieren
4. **JSON Datei** in Projekt speichern
5. **Code implementieren** (siehe Beispiel oben)
6. **Testen und verfeinern**

---

**Fragen oder Probleme?**
- Recraft.ai Tutorial: https://www.recraft.ai/blog/generate-lottie-files-with-ai-for-free
- LottieFiles Docs: https://lottiefiles.com/docs
- Tauri Splash Screen: https://v2.tauri.app/

---

**Erstellt von:** Claude Code
**Datum:** 2025-10-18
**Projekt:** DPolG Buchungssystem Modern
