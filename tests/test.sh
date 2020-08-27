#!/bin/bash

set -e

tests=()

update=0

while [ $# -gt 1 ]; do
    case "$1" in
        --update-expected|-u)
            update=1
            shift
        ;;
        --test|-t)
            shift
            tests+=($1)
            shift
        ;;
        *)
            break
        ;;
    esac
done

if [ -z "$1" ]; then
    echo "Usage: $0 [-u|--update-expected] [(-t|--test) <test-set>]... <program> [<program args>...]"
    exit 1
fi

if [ ${#tests[@]} -eq 0 ]; then
    tests=(parse parse-fail)
fi

program="$@"

testdir="$(dirname "$0")/sets"

total_passed=0
total_failed=0

pass() {
    echo -e "\r[\033[1;32mPASS\033[m]"
    ((passed++))
    ((total_passed++))
}

fail() {
    echo -e "\r[\033[1;31mFAIL\033[m]"
    ((failed++))
    ((total_failed++))
}

diff='git -c color.diff.old=green -c color.diff.new=red --no-pager diff -U1 --no-index --color --exit-code'

set +e

for dir in "${tests[@]}"; do

    passed=0
    failed=0

    if [ -d "$testdir/$dir/tests" ]; then

        outputdir="$testdir/$dir/.output"
        diffdir="$testdir/$dir/.diff"

        rm -rf "$outputdir" "$diffdir"
        mkdir "$outputdir" "$diffdir"

        if [ -f "$testdir/$dir/flags" ]; then
            flags="$(<"$testdir/$dir/flags" )"
        else
            flags=''
        fi

        for testfile in "$testdir/$dir/tests"/*; do
            testname="${testfile##*/}"
            expectedfile="$testdir/$dir/expected/$testname"
            outputfile="$outputdir/$testname"
            difffile="$diffdir/$testname"
            echo -n "[....] $dir/$testname"
            { $program $flags "$testfile"; } &> $outputfile
            r=$?
            if [ $r != 0 ]; then
                fail
                echo -e "\033[31mProgram exited with non-zero status code ($r). The output was:\033[m"
                cat "$outputfile"
                echo -e "\033[31mProgram was invoked as: $program $flags \"$testfile\"\033[m"
            elif [ $update == 1 ]; then
                pass
                cp "$outputfile" "$expectedfile"
            elif [ ! -f "$expectedfile" ]; then
                echo -e "\r[\033[1;34m????\033[m] $dir/$testname - no expected output available, got:"
                cat "$outputfile"
            elif $diff "$expectedfile" "$outputfile" > "$difffile"; then
                pass
            else
                fail
                tail -n+5 "$difffile"
            fi
        done

    else

        echo -n "[....] No such directory: $testdir/$dir/tests"
        fail

    fi

    if [ ${#tests[@]} -gt 1 ]; then
        if [ $failed -gt 0 ]; then
            echo -e "[\033[1;31m$failed $dir TEST(S) FAILED\033[m]\n";
        else
            echo -e "[\033[1;32mALL $passed $dir TESTS PASSED\033[m]\n";
        fi
    fi

done

if [ $total_failed -gt 0 ]; then
    echo -e "[\033[1;31m$total_failed TEST(S) FAILED\033[m]";
    exit 1
else
    echo -e "[\033[1;32mALL $total_passed TESTS PASSED\033[m]";
    exit 0
fi
