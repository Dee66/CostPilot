# Bash completion for costpilot
# Install: Copy to /etc/bash_completion.d/ or ~/.bash_completion
# Or source directly: source completions/costpilot.bash

_costpilot() {
    local cur prev opts base
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Top-level commands
    local commands="scan diff autofix init map slo policy audit heuristics explain policy-dsl group version help"

    # Global options
    local global_opts="--verbose --format --help --version"

    # If we're completing the first argument after costpilot
    if [ $COMP_CWORD -eq 1 ]; then
        COMPREPLY=( $(compgen -W "${commands}" -- ${cur}) )
        return 0
    fi

    # Get the subcommand
    local subcommand="${COMP_WORDS[1]}"

    case "${prev}" in
        --plan|-p)
            # Complete with .json files
            COMPREPLY=( $(compgen -f -X '!*.json' -- ${cur}) )
            return 0
            ;;
        --before|-b|--after|-a)
            # Complete with .json files
            COMPREPLY=( $(compgen -f -X '!*.json' -- ${cur}) )
            return 0
            ;;
        --policy)
            # Complete with .yaml and .yml files
            COMPREPLY=( $(compgen -f -X '!*.y@(a)ml' -- ${cur}) )
            return 0
            ;;
        --format|-f)
            COMPREPLY=( $(compgen -W "json text markdown" -- ${cur}) )
            return 0
            ;;
        --mode|-m)
            COMPREPLY=( $(compgen -W "snippet patch" -- ${cur}) )
            return 0
            ;;
    esac

    # Subcommand-specific completions
    case "${subcommand}" in
        scan)
            local opts="--plan --explain --policy --exemptions --format --fail-on-critical --autofix"
            COMPREPLY=( $(compgen -W "${opts} ${global_opts}" -- ${cur}) )
            ;;
        diff)
            local opts="--before --after"
            COMPREPLY=( $(compgen -W "${opts} ${global_opts}" -- ${cur}) )
            ;;
        autofix)
            local opts="--mode --plan --drift-safe"
            COMPREPLY=( $(compgen -W "${opts} ${global_opts}" -- ${cur}) )
            ;;
        init)
            local opts="--no-ci"
            COMPREPLY=( $(compgen -W "${opts} ${global_opts}" -- ${cur}) )
            ;;
        map)
            local opts="--format --output --show-costs --cycle-detection --depth --filter"
            COMPREPLY=( $(compgen -W "${opts} ${global_opts}" -- ${cur}) )
            ;;
        slo)
            local slo_commands="check burn"
            if [ $COMP_CWORD -eq 2 ]; then
                COMPREPLY=( $(compgen -W "${slo_commands}" -- ${cur}) )
            else
                local opts="--slo --snapshots --min-snapshots --min-r-squared"
                COMPREPLY=( $(compgen -W "${opts} ${global_opts}" -- ${cur}) )
            fi
            ;;
        policy)
            local policy_commands="submit approve reject activate deprecate status history diff"
            if [ $COMP_CWORD -eq 2 ]; then
                COMPREPLY=( $(compgen -W "${policy_commands}" -- ${cur}) )
            else
                local opts="--policy --approvers --approver --comment --reason --actor --from --to"
                COMPREPLY=( $(compgen -W "${opts} ${global_opts}" -- ${cur}) )
            fi
            ;;
        policy-dsl)
            local dsl_commands="list validate test stats example"
            if [ $COMP_CWORD -eq 2 ]; then
                COMPREPLY=( $(compgen -W "${dsl_commands}" -- ${cur}) )
            else
                local opts="--policy --verbose --format"
                COMPREPLY=( $(compgen -W "${opts} ${global_opts}" -- ${cur}) )
            fi
            ;;
        group)
            local group_commands="module service environment attribution all"
            if [ $COMP_CWORD -eq 2 ]; then
                COMPREPLY=( $(compgen -W "${group_commands}" -- ${cur}) )
            else
                local opts="--by-category --min-cost --max-groups --detailed --detect-anomalies --top-n --output"
                COMPREPLY=( $(compgen -W "${opts} ${global_opts}" -- ${cur}) )
            fi
            ;;
        heuristics)
            local heur_commands="list show update validate stats search"
            if [ $COMP_CWORD -eq 2 ]; then
                COMPREPLY=( $(compgen -W "${heur_commands}" -- ${cur}) )
            else
                local opts="--region --instance-type --resource-type --file --query"
                COMPREPLY=( $(compgen -W "${opts} ${global_opts}" -- ${cur}) )
            fi
            ;;
        explain)
            local explain_commands="prediction detection policy regression"
            if [ $COMP_CWORD -eq 2 ]; then
                COMPREPLY=( $(compgen -W "${explain_commands}" -- ${cur}) )
            else
                local opts="--resource --plan --policy --verbose"
                COMPREPLY=( $(compgen -W "${opts} ${global_opts}" -- ${cur}) )
            fi
            ;;
        version)
            local opts="--detailed"
            COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
            ;;
        *)
            COMPREPLY=( $(compgen -W "${global_opts}" -- ${cur}) )
            ;;
    esac
}

complete -F _costpilot costpilot
