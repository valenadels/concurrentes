[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-24ddc0f5d75046c5622901739e7c5dd533143b0c8e959d652212380cedb1ea36.svg)](https://classroom.github.com/a/VqwN-ppG)

## Cómo ejecutar el TP
- Descargar el data set ejecutando el script `download_dataset.sh` ubicado en el directorio raíz del proyecto.
- Desde el directorio raíz del proyecto, ejecutar el comando `cargo run -- <cant de threads>` donde `<cant de threads>` es la cantidad de threads que se desean utilizar para la ejecución del programa.
  Ejemplo: `cargo run -- 5`
- Para visualizar toda la salida del programa, se recomienda redirigir la salida a un archivo de texto. Ejemplo: `cargo run -- 5 > output.txt`

## Cómo ejecutar los tests
- Desde el directorio raíz del proyecto, ejecutar el comando `cargo test`. Esto ejecutará todos los tests del proyecto, tanto los unitarios como los de integración.
