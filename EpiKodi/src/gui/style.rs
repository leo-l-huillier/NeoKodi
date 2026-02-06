pub const GLOBAL_STYLE: &str = r#"
    /* RESET GLOBAL */
    html, body { 
        margin: 0; 
        padding: 0;
        font-family: 'Segoe UI', sans-serif; 
        background-color: #121212; 
        color: white; 
        
        /* 👇 ON BLOQUE LE CORPS DE PAGE */
        width: 100vw;
        height: 100vh;
        overflow: hidden; /* C'est le container qui scrollera, pas le body */
    }
    
    * { box-sizing: border-box; }
    a { text-decoration: none; color: inherit; }

    /* CONTENEUR PRINCIPAL */
    .container { 
        display: flex; 
        flex-direction: column;
        
        /* 👇 LA CORRECTION EST ICI : 100vh FORCE LA TAILLE DE L'ÉCRAN */
        height: 100vh;
        width: 100%;
        
        /* C'est ici qu'on active le scroll */
        overflow-y: auto;
        overflow-x: hidden;
        
        padding: 0; 
    }

    /* --- SCROLLBAR PERSONNALISÉE (Pour être sûr qu'on la voit) --- */
    /* WebKit (Chrome, Edge, WebView2) */
    ::-webkit-scrollbar {
        width: 14px; /* Un peu plus large pour être visible */
    }
    ::-webkit-scrollbar-track {
        background: #0a0a0a; 
        border-left: 1px solid #333;
    }
    ::-webkit-scrollbar-thumb {
        background: #444; 
        border-radius: 7px;
        border: 2px solid #0a0a0a; /* Petit bord pour faire joli */
    }
    ::-webkit-scrollbar-thumb:hover {
        background: #007acc; /* Devient bleu au survol */
    }

    /* --- BARRE DU HAUT (STICKY) --- */
    .top-bar {
        position: sticky; 
        top: 0;
        z-index: 100;
        background-color: #121212; /* Opaque pour cacher le contenu qui passe dessous */
        
        display: flex;
        justify-content: center;
        align-items: center;
        
        padding: 20px;
        margin-bottom: 20px;
        border-bottom: 1px solid #333;
        min-height: 80px;
        flex-shrink: 0;
        box-shadow: 0 4px 15px rgba(0,0,0,0.8);
    }

    .page-title {
        font-size: 2rem;
        font-weight: bold;
        text-align: center;
        text-transform: uppercase;
        letter-spacing: 2px;
        text-shadow: 0 2px 4px rgba(0,0,0,0.5);
    }

    .btn-nav {
        position: absolute;
        left: 20px;
        top: 50%;
        transform: translateY(-50%);
        background-color: #252525;
        color: #aaa;
        padding: 10px 20px;
        border-radius: 8px;
        font-weight: bold;
        transition: 0.2s;
        border: 1px solid #333;
        cursor: pointer;
        display: flex; align-items: center; gap: 10px;
        z-index: 10;
    }
    .btn-nav:hover {
        background-color: #007acc;
        color: white;
        border-color: #007acc;
    }

    /* --- CONTENU --- */
    .media-grid, .audio-list {
        padding: 20px; 
        padding-bottom: 100px; /* Grosse marge en bas pour scroller confortablement */
    }

    /* --- GRILLE --- */
    .media-grid { 
        display: grid; 
        /* Responsive intelligent */
        grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); 
        gap: 25px; 
    }

    .media-card { 
        background: #1e1e1e; 
        border-radius: 12px; 
        padding: 20px;
        display: flex; 
        flex-direction: column;
        align-items: center; 
        justify-content: center; 
        transition: transform 0.2s, background-color 0.2s;
        box-shadow: 0 4px 6px rgba(0,0,0,0.3);
        border: 1px solid #333;
        cursor: pointer;
        min-height: 180px; 
        text-align: center;
    }

    .media-card:hover {
        transform: translateY(-5px);
        background-color: #2d2d2d;
        border-color: #007acc;
    }

    .card-icon { font-size: 3rem; margin-bottom: 15px; }
    .card-text { font-size: 1.1rem; font-weight: 600; }

    /* --- LISTE AUDIO --- */
    .audio-list { display: flex; flex-direction: column; gap: 10px; }
    
    .audio-row {
        display: flex; align-items: center;
        background-color: #1e1e1e; padding: 12px 20px;
        border-radius: 8px; cursor: pointer;
        border: 1px solid transparent; transition: 0.2s;
    }
    .audio-row:hover { background-color: #2d2d2d; border-color: #444; transform: translateX(5px); }
    
    .audio-icon { 
        width: 45px; height: 45px; 
        background: #333; border-radius: 6px; 
        margin-right: 20px; display: flex; align-items: center; justify-content: center; 
        font-size: 1.5rem; 
    }
    .audio-info { flex: 1; }
    .audio-title { font-weight: 600; }
    .audio-artist { font-size: 0.85rem; color: #888; }
    
    .loading-container {
        width: 100%;
        max-width: 500px;
        background-color: #333;
        border-radius: 4px;
        height: 6px;
        overflow: hidden;
        margin-top: 20px;
        position: relative;
    }

    .loading-bar {
        height: 100%;
        background-color: #007acc;
        width: 50%;
        position: absolute;
        animation: loading 1.5s infinite ease-in-out;
        border-radius: 4px;
    }

    @keyframes loading {
        0% { left: -50%; width: 30%; }
        50% { width: 60%; }
        100% { left: 100%; width: 30%; }
    }
"#;