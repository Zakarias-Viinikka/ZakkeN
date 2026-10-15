#!/usr/bin/env bash
set -e
ORIGINAL_CWD="$(pwd)"
cd "$(dirname "$0")"

# These are all the projects in ZakkeN that have android builds, and the
# specific crates that produce android output:
#
#   yrs/build_android.sh
#     - crate: yrs (lib name my_yrs_lib)
#     - output: android_output_folder/
#
#   love_letter/build_android.sh
#     - crate: love_letter
#     - output: android_output_folder/
#
#   text_diff/build_android.sh
#     - crate: text_diff
#     - output: android_output_folder/
#
#   client_table_blueprints/build_android.sh
#     - crate: client_table_blueprints
#     - output: android_output/
#
#   cqrs_state_synchronization/data_builder_for_operations_that_need_to_be_correct/build_android.sh
#     - crate: data_builder_for_operations_that_need_to_be_correct
#     - output: android_output/
#
# z_db's protocol and db_wrapper also produce android output, but they
# live in a different repo and are built by z_db/build_web_and_android.sh.

./yrs/build_android.sh
./love_letter/build_android.sh
./text_diff/build_android.sh
./client_table_blueprints/build_android.sh
./cqrs_state_synchronization/data_builder_for_operations_that_need_to_be_correct/build_android.sh

echo "all android builds done"

# Only reached if every build above succeeded (set -e at the top).
# Copy the fresh outputs into the android project.
echo "copying fresh outputs into android_app..."
(cd android_app && ./update.sh)

cd "$ORIGINAL_CWD"
