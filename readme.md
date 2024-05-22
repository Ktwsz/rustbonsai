# Jak używać
Aby uruchomić drzewko należy użyć komendy *cargo run*


- -s, --seed   &emsp;&emsp;&emsp;&ensp; Podaj wartość typu u64 
- -l, --live    &emsp;&emsp;&emsp;&emsp;&ensp; Wyświetl animacje rysowania drzewa
- -p, --particles  &emsp;&emsp;    Wyświetl spadające liście
- -t, --theme   &emsp;&emsp;&emsp;&ensp;Zmiana koloru drzewa: [default: 1]
  - 1 Podstawowe, 
  - 2 Wiśnia, 
  - 3 Klon, 
  - 4 Avatar 
- -h, --help   &emsp;&emsp;&emsp;&emsp;&ensp;Wyświetl pomoc
- -V, --version       &emsp;&emsp;&emsp; Wyświetlanie wersji 

# Linki:
- implementacja cbonsai: https://gitlab.com/jallbrit/cbonsai/-/blob/master/cbonsai.c?ref_type=heads
- ratatui: https://github.com/ratatui-org/ratatui
- ratatui canvas: https://github.com/ratatui-org/ratatui/blob/main/examples/canvas.rs#L33
- inna implementacja bonsai: https://github.com/haozoo/bonsai/blob/main/src/bonsai/bonsai.h
