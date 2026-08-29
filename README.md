{
  "diagrama": "Flujo de Autenticación y Eliminación en Gmail con Rust",
  "fases": [
    {
      "nombre": "FASE 1: AUTENTICACIÓN OAUTH 2.0",
      "nodos": [
        { "id": "f1_n1", "label": "Leer credentials.json", "tipo": "entrada_datos" },
        { "id": "f1_n2", "label": "Configurar Cliente HTTPS", "tipo": "proceso" },
        { "id": "f1_n3", "label": "Iniciar Flujo OAuth 2.0", "tipo": "seguridad" },
        { "id": "f1_n4", "label": "Login en el Navegador (Google)", "tipo": "accion_usuario" },
        { "id": "f1_n5", "label": "Guardar Token en token.json", "tipo": "almacenamiento" },
        { "id": "f1_n6", "label": "Instanciar Cliente Gmail API", "tipo": "proceso" }
      ],
      "conexiones": [
        { "desde": "f1_n1", "hacia": "f1_n2" },
        { "desde": "f1_n2", "hacia": "f1_n3" },
        { "desde": "f1_n3", "hacia": "f1_n4" },
        { "desde": "f1_n4", "hacia": "f1_n5" },
        { "desde": "f1_n5", "hacia": "f1_n6" }
      ]
    },
    {
      "nombre": "FASE 2: INICIALIZACIÓN Y BÚSQUEDA",
      "nodos": [
        { "id": "f2_n1", "label": "Conectar con Gmail Cloud", "tipo": "red" },
        { "id": "f2_n2", "label": "Obtener Perfil de Usuario ('me')", "tipo": "api_call" },
        { "id": "f2_n3", "label": "Mostrar Cuenta Activa en Consola", "tipo": "salida_pantalla" },
        { "id": "f2_n4", "label": "Usuario escribe Query (Búsqueda)", "tipo": "entrada_usuario" },
        { "id": "f2_n5", "label": "Ejecutar: messages_list()", "tipo": "api_call" },
        { "id": "f2_dec1", "label": "¿Hay resultados?", "tipo": "decision" }
      ],
      "conexiones": [
        { "desde": "f1_n6", "hacia": "f2_n1" },
        { "desde": "f2_n1", "hacia": "f2_n2" },
        { "desde": "f2_n2", "hacia": "f2_n3" },
        { "desde": "f2_n3", "hacia": "f2_n4" },
        { "desde": "f2_n4", "hacia": "f2_n5" },
        { "desde": "f2_n5", "hacia": "f2_dec1" },
        { "desde": "f2_dec1", "hacia": "fin_programa", "condicion": "NO" },
        { "desde": "f2_dec1", "hacia": "f3_n1", "condicion": "SÍ" }
      ]
    },
    {
      "nombre": "FASE 3: CONFIRMACIÓN Y ELIMINACIÓN",
      "nodos": [
        { "id": "f3_n1", "label": "Obtener IDs de Mensajes", "tipo": "proceso" },
        { "id": "f3_n2", "label": "Pedir Confirmación en Pantalla", "tipo": "salida_pantalla" },
        { "id": "f3_dec1", "label": "¿Usuario confirma? [y/N]", "tipo": "decision" },
        { "id": "f3_n3", "label": "Ejecutar: messages_trash()", "tipo": "api_call" },
        { "id": "f3_n4", "label": "Mover a la Papelera Gmail", "tipo": "accion_cloud" }
      ],
      "conexiones": [
        { "desde": "f3_n1", "hacia": "f3_n2" },
        { "desde": "f3_n2", "hacia": "f3_dec1" },
        { "desde": "f3_dec1", "hacia": "fin_programa", "condicion": "NO" },
        { "desde": "f3_dec1", "hacia": "f3_n3", "condicion": "SÍ" },
        { "desde": "f3_n3", "hacia": "f3_n4" },
        { "desde": "f3_n4", "hacia": "fin_programa" }
      ]
    }
  ],
  "nodos_globales": [
    { "id": "fin_programa", "label": "FIN (0)", "tipo": "terminador" }
  ]
}
