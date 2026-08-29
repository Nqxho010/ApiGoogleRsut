┌─────────────────────────────────────────────────────────┐
 │               [ FASE 1: AUTENTICACIÓN OAUTH 2.0 ]       │
 └───────────────────────────┬─────────────────────────────┘
                             ▼
                 📂 Leer 'credentials.json'
                             │
                             ▼
                 🌐 Configurar Cliente HTTPS
                             │
                             ▼
                 🔐 Iniciar Flujo OAuth 2.0
                             │
                             ▼
                 🌐 Login en el Navegador (Google)
                             │
                             ▼
                 💾 Guardar Token en 'token.json'
                             │
                             ▼
                 🛠️  Instanciar Cliente Gmail API
                             │
                             ▼
 ┌─────────────────────────────────────────────────────────┐
 │               [ FASE 2: INICIALIZACIÓN Y BÚSQUEDA ]     │
 └───────────────────────────┬─────────────────────────────┘
                             ▼
                 🔌 Conectar con Gmail Cloud
                             │
                             ▼
                 👤 Obtener Perfil de Usuario ("me")
                             │
                             ▼
                 📧 Mostrar Cuenta Activa en Consola
                             │
                             ▼
                 ⌨️  Usuario escribe Query (Búsqueda)
                             │
                             ▼
                 🔍 Ejecutar: messages_list()
                             │
                             ▼
                    ❓ ¿Hay resultados?
                           /     \
                         NO       SÍ
                        /           \
                       ▼             ▼
                 ┌───────────┐   🆔 Obtener IDs de Mensajes
                 │  FIN (0)  │       │
                 └───────────┘       ▼
 ┌───────────────────────────────────┴─────────────────────┐
 │               [ FASE 3: CONFIRMACIÓN Y ELIMINACIÓN ]     │
 └───────────────────────────┬─────────────────────────────┘
                             ▼
                 ⚠️  Pedir Confirmación en Pantalla
                             │
                             ▼
                     ❓ ¿Usuario confirma? [y/N]
                           /     \
                         NO       SÍ
                        /           \
                       ▼             ▼
                 ┌───────────┐   🗑️  Ejecutar: messages_trash()
                 │  FIN (0)  │       │
                 └───────────┘       ▼
                                 🚀 Mover a la Papelera Gmail
                                     │
                                     ▼
                               ┌───────────┐
                               │  FIN (0)  │
                               └───────────┘
