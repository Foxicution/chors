#!/bin/bash

# Function to print background colors in the terminal with their ratatui names
print_background_color_block_with_names() {
  declare -A color_names=(
    [0]="Black"
    [1]="Red"
    [2]="Green"
    [3]="Yellow"
    [4]="Blue"
    [5]="Magenta"
    [6]="Cyan"
    [7]="White"
    [8]="Bright Black"
    [9]="Bright Red"
    [10]="Bright Green"
    [11]="Bright Yellow"
    [12]="Bright Blue"
    [13]="Bright Magenta"
    [14]="Bright Cyan"
    [15]="Bright White"
  )

  for i in {0..7}; do
    for j in {0..7}; do
      color_code=$((i * 8 + j))
      if [ $color_code -lt 16 ]; then
        printf "\033[48;5;${color_code}m%4d %-15s " ${color_code} "${color_names[$color_code]}"
      else
        printf "\033[48;5;${color_code}m%4d %-15s " ${color_code} ""
      fi
    done
    echo ""
  done
  echo -e "\033[0m"  # Reset to default colors
}

# Print 16 basic background colors
echo "16 Basic Background Colors:"
print_background_color_block_with_names

# Print 256 background colors if the terminal supports it
if [ "$TERM" != "linux" ]; then
  echo "256 Background Colors:"
  for i in {0..5}; do
    for j in {0..5}; do
      for k in {0..5}; do
        color_code=$((16 + i * 36 + j * 6 + k))
        printf "\033[48;5;${color_code}m%4d %-15s " ${color_code} ""
      done
      echo ""
    done
  done

  # Grayscale background colors
  echo "Grayscale Background Colors:"
  for i in {232..255}; do
    printf "\033[48;5;${i}m%4d %-15s " ${i} ""
    [ $((($i - 231) % 6)) == 4 ] && echo ""
  done
  echo -e "\033[0m"  # Reset to default colors
fi

