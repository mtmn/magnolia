#!/bin/zsh
magnolia_zle_change_to_file() {
        magnolia_change_to_file && zle reset-prompt 2>/dev/null || true
}

magnolia_zle_change_to_dir() {
        magnolia_change_to_dir && zle reset-prompt 2>/dev/null || true
}

magnolia_zle_custom_dirs() {
        magnolia_custom_dirs && zle reset-prompt 2>/dev/null || true
}

zle -N magnolia_zle_change_to_file
zle -N magnolia_zle_change_to_dir
zle -N magnolia_zle_custom_dirs

bindkey '^F' 'magnolia_zle_change_to_file'
bindkey '^K' 'magnolia_zle_change_to_dir'
bindkey '^G' 'magnolia_zle_custom_dirs'
