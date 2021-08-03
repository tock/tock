#!/usr/bin/env bash


OBJDUMP_OUT=~/dump9 #output of riscv64-unknown-elf-onjdump -d ELF_FILE
ELF_FILE=~/elf9 # ELF_FILE of Tock kernel (unstripped)

# A list of functions in the Ti50 Tock kernel for which a call to this function
# guarantees a panic
declare -a funcs=("expect_failed" # This is an option function
                  "unwrap_failed" # This is a result function
                  "panic_bounds_check"
                  "len_mismatch_fail"
                  "slice_index_order_fail"
                  "slice_start_index_len_fail"
                  "slice_end_index_len_fail"
                  "slice_error_fail" #str::slice_error_fail
                  "core9panicking5panic"
                  "core..fmt..builders..PadAdapter" #calls slice_error_fail
                  "11copy_within17" # calls panicking::panic
                  "write_char" # calls PadAdapter one above
                  "write_str" # calls write_char
                  "printable5check" #calls slice_index_order_fail
                  'char$u20$as$u20$core..fmt..Debug' #calls printable5check
                  )

# returns 0 if the passed origin string matches one of the known panic
# functions above
check_if_core_match () {
  for func in "${funcs[@]}"; do
    if [[ $1  == *"$func"* ]]; then
      return 0
    fi
  done
  return 1
}

# global var for number of calls to potential panics in ti50 code
PANIC_COUNT=0
PANIC_COUNT_AT_START=0
LAST_FUNC=
for func in "${funcs[@]}"; do
  PANIC_COUNT_LAST_FUNC=$(( PANIC_COUNT - PANIC_COUNT_AT_START ))
  PANIC_COUNT_AT_START=$(( PANIC_COUNT ))
  echo total calls to "$LAST_FUNC": "$PANIC_COUNT_LAST_FUNC"
  LAST_FUNC="$func"
  # get list of addresses of instructions that call these functions
  ADDRS=$(grep "$func" $OBJDUMP_OUT | awk '{print 0"x"$1}' | grep ^.*:$ | sed 's/.$//')
  NUM_ADDRS=$(echo "$ADDRS" | wc -l)
  echo calls to "$func":
  #printf %s "$ADDRS" |
  while IFS= read -r addr; do
    DWARFDUMP=$(llvm-dwarfdump-11 --lookup="$addr" $ELF_FILE)
    DECL_FILE=$(echo "$DWARFDUMP" | grep -A 1 'DW_AT_decl_file')
    CALL_FILE=$(echo "$DWARFDUMP" | grep -A 1 'DW_AT_call_file')
    LINE_INFO=$(echo "$DWARFDUMP" | grep 'Line info:')
    if [ -z "$DECL_FILE" ] && [ -z "$CALL_FILE" ]; then
      if [ -z "$LINE_INFO" ]; then
        echo "$DWARFDUMP"
        echo "MISSING INFO"
        continue # TODO remove
        exit 1
      fi
      echo "$addr":
      PANIC_COUNT=$(( PANIC_COUNT + 1 ))
      echo "$LINE_INFO"
    elif [ ! -z "$DECL_FILE" ] && [ ! -z "$CALL_FILE" ]; then
      echo "BOTH DECL AND CALL"
      exit 1 # have not seen this happen
    elif [ ! -z "$DECL_FILE" ]; then
      if [[ "$DECL_FILE"  == *"/core/"* ]]; then
        ORIGIN=$(echo "$DWARFDUMP" | grep 'DW_AT_abstract_origin')
        if [ -z "$ORIGIN" ]; then
          if [[ "$DECL_FILE" == *"core/src/str/mod.rs"* ]] && [[ "$DECL_FILE" == *"(83)"* ]]; then
            # the list above
            continue
          fi
          if [[ "$DECL_FILE" == *"core/src/fmt/builders.rs"* ]] && [[ "$DECL_FILE" == *"34"* ]]; then
            # the list above
            continue
          fi
          if [[ "$DECL_FILE" == *"core/src/slice/mod.rs"* ]] && [[ "$DECL_FILE" == *"3089"* ]]; then
            # the list above (copy_within)
            continue
          fi
          if [[ "$DECL_FILE" == *"core/src/fmt/mod.rs"* ]] && [[ "$DECL_FILE" == *"160"* ]]; then
            # the list above (write_char)
            continue
          fi
          if [[ "$DECL_FILE" == *"core/src/fmt/mod.rs"* ]] && [[ "$DECL_FILE" == *"194"* ]]; then
            # the list above (write_char, again)
            continue
          fi
          if [[ "$DECL_FILE" == *"core/src/fmt/mod.rs"* ]] && [[ "$DECL_FILE" == *"190"* ]]; then
            # the list above (write_str)
            continue
          fi
        fi
        echo "NEVER HAPPENS"
        echo "$ORIGIN"
        echo "$DECL_FILE"
        echo "$LINE_INFO"
        exit 1
      fi
      echo "$addr":
      PANIC_COUNT=$(( PANIC_COUNT + 1 ))
      echo "$DECL_FILE"
    elif [ ! -z "$CALL_FILE" ]; then
      if [[ "$CALL_FILE"  == *"/core/"* ]]; then
        ORIGIN=$(echo "$DWARFDUMP" | grep 'DW_AT_abstract_origin')
        if [ -z "$ORIGIN" ]; then
          echo "NO ORIGIN"
          exit 1
        elif [[ "$ORIGIN" == *"closure"* ]]; then
          #echo "$ORIGIN"
          # Probably on the line in LINE_INFO
          if [[ "$ORIGIN" == *"4core7"* ]]; then
            echo "Panic in closure, probably here: "
            echo "ORIGIN STILL IN CORE"
            exit 1
          else
            echo "$addr":
            PANIC_COUNT=$(( PANIC_COUNT + 1 ))
            echo "Panic in closure, probably here: "
            echo "$LINE_INFO"
          fi
        else
          if [[ "$ORIGIN" == *"core"* ]]; then
            #if check_if_core_match "$ORIGIN"; then
            #  echo "$ORIGIN"
            #  echo "ALREADY COUNTED" #TODO: is this logic correct?
            if [[ "$ORIGIN" == *"_ZN4core5slice5index5range17h8489d274a"* ]]; then
              #print nothing, this particular example is just called by copy_within
              continue
            elif [[ "$ORIGIN" == *"core..slice..index..SliceIndex"* ]]; then
              #print nothing, this particular example is just called by copy_within
              continue
            elif [[ "$CALL_FILE" == *"/core/src/unicode/printable.rs"* ]] && [[ "$CALL_FILE" == *"(26)"* ]]; then
              #print nothing, this particular example is just called by unicode printable5check
              continue
            elif [[ "$CALL_FILE" == *"/core/src/char/methods.rs"* ]] && [[ "$CALL_FILE" == *"(422)"* ]]; then
              #print nothing, this particular example is just called by copy_within slice_error_fail
              continue
            elif [[ "$CALL_FILE" == *"/core/src/str/mod.rs"* ]] && [[ "$CALL_FILE" == *"112"* ]]; then
              #print nothing, this particular example is just called by copy_within slice_error_fail
              continue
            elif [[ "$ORIGIN" == *"_ZN4core7unicode12unicode_data11skip_search17hda"* ]]; then
              #print nothing, this particular example is currently just called from within slice_error_fail
              continue
            else
              # lets check parent
              DWARFDUMP_PARENT=$(llvm-dwarfdump-11 --lookup="$addr" -p --parent-recurse-depth=1 $ELF_FILE)
              # just get first match, it will be above the child match
              PARENT_MATCH=$(echo "$DWARFDUMP_PARENT" | grep -A 1 -m 1 '\(DW_AT_decl_file\|DW_AT_call_file\)')
              if [[ "$PARENT_MATCH" == *"core"* ]]; then
                # TODO: Are multiple parents possible?
                DWARFDUMP_PARENT2=$(llvm-dwarfdump-11 --lookup="$addr" -p --parent-recurse-depth=2 $ELF_FILE)
                # just get first match, it will be above the child match
                PARENT_MATCH2=$(echo "$DWARFDUMP_PARENT2" | grep -A 1 -m 1 '\(DW_AT_decl_file\|DW_AT_call_file\)')
                if [[ "$PARENT_MATCH2" == *"core"* ]]; then
                  if [[ "$PARENT_MATCH2" == *"/core/src/fmt/builders.rs"*"(14"* ]]; then
                    echo "$addr":
                    PANIC_COUNT=$(( PANIC_COUNT + 1 ))
                    echo "DERIVE(DEBUG) generated"
                    continue
                  fi
                  echo "AND PARENT2 STILL IN CORE"
                  echo "PARENT2:"
                  echo "$PARENT_MATCH2"
                  echo "PARENT:"
                  echo "$PARENT_MATCH"
                  echo "$ORIGIN"
                  echo "$CALL_FILE"
                  echo "$LINE_INFO"
                  exit 1
                else
                  echo "$addr":
                  PANIC_COUNT=$(( PANIC_COUNT + 1 ))
                  echo "FOUND PANIC IN PARENT2:"
                  echo "$PARENT_MATCH2"
                fi
              else
                echo "$addr":
                PANIC_COUNT=$(( PANIC_COUNT + 1 ))
                echo "FOUND PANIC IN PARENT:"
                echo "$PARENT_MATCH"
              fi
            fi
          else
            echo "$ORIGIN"
            echo "$CALL_FILE"
            echo "ORIGIN NOT IN CORE, PANIC FOUND?"
            exit 1
          fi
        fi
      else
        # found source, outside core, immediately
        echo "$addr":
        PANIC_COUNT=$(( PANIC_COUNT + 1 ))
        echo "$CALL_FILE"
      fi
    fi
    echo
  done <<< $(printf %s "$ADDRS")
  #DWARFDUMP=$(echo "$ADDRS" | xargs -i llvm-dwarfdump-11 --lookup={} $ELF_FILE)
  #echo "$DWARFDUMP" | grep -A 1 '\(DW_AT_decl_file\|DW_AT_call_file\)'
done

echo $PANIC_COUNT
