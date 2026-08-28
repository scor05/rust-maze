# RUST-GLOBAL OFFENSIVE

Este es un juego hecho con Rust y su librería de gráficas Raylib que trata de imitar al famoso CSGO con gráficas de Wolfenstein 1.

---
## Video de funcionamiento/demostración

El video demostrando cómo funciona el juego se puede encontrar [en este enlace](https://youtu.be/_WslzFJKgvM).

---
## Cómo jugar

El juego está configurado para que se pueda correr `cargo run` desde la carpeta raíz (por ende las únicas dependencias necesarias para el juego son rust 
y cargo). Si al ejecutar ese comando no se lanza el juego correctamente y muestra errores de fallas al cargar texturas/audios, lo más probable es que no 
se ejecutó desde la carpeta raíz. 

---
## Descripción

Este juego funciona exactamente como funciona actualmente CS2 pero sin el elemento multiplayer. Para cada uno de los mapas que implementé
hay dos bombsites, los cuales el jugador tiene que llegar a ellos y defusar para ganar. También hay varios enemigos que están protegiendo los sites,
por lo que el jugador debe de tener cuidado.

En la esquina superior derecha del juego se encuentra el minimapa, el cual tiene la siguiente leyenda:
    - Pixeles beige: paredes
    - Pixeles amarillos: bombsites
    - Pixeles rojos: enemigos
    - Pixeles cafés: cajas
    - Pixeles grises: suelo
    - Pixeles azules/celestes: jugador
Con base a la información que provee el minimapa, el jugador debe traversar el mapa real y llegar a ambos bombsites para defusarlos.

El jugador también está equipado con la clasiquísima deagle (Desert Eagle), la cual puede matar a enemigos de una bala pero con la consecuencia
de solo tener 7 balas por cargador y 35 en reserva. 

---
## Interfaz

Al lanzar el juego se verá la interfaz clásica del CS:GO de antes, en la cual hay varios botones: 
    - "Play" para ingresar al juego (si no se ha seleccionado un mapa en la página de map select, el default se considera inferno)
    - "Map Select" para seleccionar el mapa que se quiere jugar y a la vez ver las instrucciones
    - "Exit Game" para salir del juego

En la pantalla de Map Select se muestran 4 mapas en forma de tarjeta con su foto respectiva, los cuales al darle click se cambiará el mapa actualmente seleccionado. 
Si se selecciona la tarjeta de mapa que dice "Random", un mapa será seleccionado al azar y se usará ese.
Para regresar al menú principal solo se debe de presionar el botón de Back.

Ya estando en el juego y habiendo defusado ambos bombsites, se mostrará la pantalla de victoria y desde ella se puede regresar al menú
principal y seleccionar otro mapa o entrar otra vez al juego.

---
## Controles

Flechas derecha/izquierda -> rotar la cámara
Flechas arriba/abajo -> mover personaje hacia el frente o atrás
WASD -> movimiento
Movimiento del mouse -> mover cámara horizontalmente
Click izquierdo -> disparar
E -> defusar (solo si está el jugador parado en un bombsite)
R -> recargar
Shift izquierdo -> sprint
Q -> regresar a menú principal (estando dentro del juego)
