This command line utility made in Rust by Gemini reads cleans csvs in two cases:

when only a file is passed it checks if there is duplicated emails in email column of a csv file

if two files are passed, first is considered the file that must not have in second file and It generates a third file that contents of file one does not contain on file two.
