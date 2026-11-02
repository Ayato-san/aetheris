#!/usr/bin/env bash

# Shared by the build entry points. Parse data rather than executing .env as shell
# code; only the public build configuration is exported to Cargo and its children.
aetheris_load_build_env() {
    local env_file="$1"
    local line key value line_number=0
    local assignment='^[[:space:]]*(export[[:space:]]+)?(AETHERIS_PUBLIC_KEY|AETHERIS_REPOSITORY)[[:space:]]*=(.*)$'
    local double_quoted='^"([^"]*)"[[:space:]]*(#.*)?$'
    local single_quoted="^'([^']*)'[[:space:]]*(#.*)?$"

    # The signing key is needed by the packaging step, not by compilation.
    unset AETHERIS_SIGNING_KEY
    [[ -f "$env_file" ]] || return 0

    while IFS= read -r line || [[ -n "$line" ]]; do
        line_number=$((line_number + 1))
        line="${line#$'\xef\xbb\xbf'}"
        line="${line%$'\r'}"
        [[ "$line" =~ $assignment ]] || continue
        key="${BASH_REMATCH[2]}"
        value="${BASH_REMATCH[3]}"
        value="${value#"${value%%[![:space:]]*}"}"
        value="${value%"${value##*[![:space:]]}"}"

        case "$value" in
            \"*)
                if [[ ! "$value" =~ $double_quoted ]]; then
                    printf 'Erreur dans .env (ligne %s) : guillemets invalides pour %s.\n' "$line_number" "$key" >&2
                    return 1
                fi
                value="${BASH_REMATCH[1]}"
                ;;
            \'*)
                if [[ ! "$value" =~ $single_quoted ]]; then
                    printf 'Erreur dans .env (ligne %s) : guillemets invalides pour %s.\n' "$line_number" "$key" >&2
                    return 1
                fi
                value="${BASH_REMATCH[1]}"
                ;;
            *)
                value="${value%%[[:space:]]#*}"
                value="${value%"${value##*[![:space:]]}"}"
                ;;
        esac
        [[ -n "$value" ]] || continue
        case "$key" in
            AETHERIS_PUBLIC_KEY)
                if [[ ! "$value" =~ ^[[:xdigit:]]{64}$ ]]; then
                    printf 'Erreur dans .env (ligne %s) : AETHERIS_PUBLIC_KEY doit contenir 64 caractères hexadécimaux.\n' "$line_number" >&2
                    return 1
                fi
                ;;
            AETHERIS_REPOSITORY)
                if [[ ! "$value" =~ ^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$ ]]; then
                    printf 'Erreur dans .env (ligne %s) : AETHERIS_REPOSITORY doit suivre le format propriétaire/dépôt.\n' "$line_number" >&2
                    return 1
                fi
                ;;
        esac
        printf -v "$key" '%s' "$value"
        export "$key"
    done < "$env_file"
}
