// src/front/style.rs

pub const GLOBAL_STYLE: &str = r#"
    body { margin: 0; font-family: sans-serif; background-color: #1a1a1a; color: white; }
    .container { display: flex; height: 100vh; }
    
    /* Sidebar */
    .sidebar { 
        width: 250px; 
        background-color: #252525; 
        display: flex; 
        flex-direction: column; 
        padding: 20px;
    }
    .nav-item {
        padding: 15px; 
        text-decoration: none; 
        color: #aaa; 
        font-size: 1.2rem;
        transition: 0.2s;
        border-radius: 8px;
        margin-bottom: 5px;
    }
    .nav-item:hover, .nav-item.router-link-active {
        background-color: #007acc;
        color: white;
    }

    /* Content */
    .content { flex: 1; padding: 40px; overflow-y: auto; }
    
    /* Grille médias */
    .media-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); gap: 20px; margin-top: 20px; }
    .media-card { background: #333; height: 200px; display: flex; align-items: center; justify-content: center; border-radius: 10px; }

    /* --- STYLE AUDIO SPECIFIQUE --- */

    /* Conteneur liste verticale */
    .audio-list {
        display: flex;
        flex-direction: column;
        gap: 8px; /* Espace entre les pistes */
        margin-top: 20px;
    }

    /* Une ligne (une chanson) */
    .audio-row {
        display: flex;
        align-items: center;
        background-color: #2a2a2a;
        padding: 10px 15px;
        border-radius: 6px;
        transition: background-color 0.2s, transform 0.1s;
        cursor: pointer;
    }

    .audio-row:hover {
        background-color: #333;
        transform: translateX(5px); /* Petit effet de mouvement vers la droite */
    }

    /* Carré pour l'icône ou la pochette à gauche */
    .audio-icon {
        width: 40px;
        height: 40px;
        background-color: #444; /* Gris par défaut */
        border-radius: 4px;
        margin-right: 15px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 1.2rem;
    }

    /* Infos texte */
    .audio-info {
        flex: 1; /* Prend toute la place disponible */
        display: flex;
        flex-direction: column;
    }

    .audio-title {
        font-weight: 600;
        color: white;
        font-size: 1rem;
    }

    .audio-artist {
        font-size: 0.85rem;
        color: #aaa;
        margin-top: 2px;
    }

    /* Durée à droite */
    .audio-duration {
        font-family: monospace;
        color: #888;
        font-size: 0.9rem;
"#;