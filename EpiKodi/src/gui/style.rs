pub const GLOBAL_STYLE: &str = r#"
    /* RESET */
    html, body { 
        margin: 0; 
        padding: 0;
        font-family: 'Segoe UI', sans-serif; 
        background-color: #121212; 
        color: white; 
        
        /* 👇 ON UTILISE % AU LIEU DE VH */
        height: 100%; 
        width: 100%;
        overflow: hidden; 
    }
    
    * { box-sizing: border-box; }
    a { text-decoration: none; color: inherit; }

    /* CONTENEUR PRINCIPAL */
    .container { 
        display: flex; 
        flex-direction: column;
        
        /* Prend 100% de la fenêtre allouée par l'OS */
        height: 100%;       
        width: 100%;
        overflow-y: auto;    
        
        padding: 20px;
        padding-bottom: 50px; /* Petit espace de sécurité en bas pour le scroll */
    }

    /* --- BARRE DU HAUT --- */
    .top-bar {
        position: relative;
        display: flex;
        justify-content: center;
        align-items: center;
        margin-bottom: 30px;
        padding-bottom: 15px;
        border-bottom: 1px solid #333;
        min-height: 60px;
        flex-shrink: 0;
    }

    .page-title {
        font-size: 2rem;
        font-weight: bold;
        text-align: center;
        text-transform: uppercase;
        letter-spacing: 2px;
    }

    .btn-nav {
        position: absolute;
        left: 0;
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

    /* --- GRILLE --- */
    .media-grid { 
        display: grid; 
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
"#;