use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path; // Adicionado para trabalhar com caminhos

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Lê os argumentos da linha de comando
    let args: Vec<String> = env::args().collect();

    match args.len() {
        3 => {
            // Modo 1: Limpar final.csv com base em sent.csv
            let sent_file_path = &args[1];
            let final_file_path = &args[2];
            let temp_output_path = format!("{}.temp", final_file_path);

            println!("Modo: Limpar '{}' usando emails de '{}'", final_file_path, sent_file_path);
            println!("Lendo emails do arquivo: {}", sent_file_path);

            let sent_emails = match read_emails_to_set(sent_file_path) {
                Ok(emails) => {
                    println!("{} emails únicos lidos de {}", emails.len(), sent_file_path);
                    emails
                }
                Err(e) => {
                    eprintln!("Erro ao ler {}: {}", sent_file_path, e);
                    return Err(e);
                }
            };

            println!("Processando e limpando o arquivo: {}", final_file_path);
            match clean_file_by_emails(&sent_emails, final_file_path, &temp_output_path) {
                Ok(count) => {
                    println!("{} linhas mantidas em {}", count, final_file_path);
                    // Substitui o arquivo original pelo temporário
                    std::fs::rename(&temp_output_path, final_file_path)?;
                    println!("Arquivo {} limpo com sucesso.", final_file_path);
                }
                Err(e) => {
                    eprintln!("Erro ao limpar {}: {}", final_file_path, e);
                    // Tenta remover o arquivo temporário em caso de erro
                    let _ = std::fs::remove_file(&temp_output_path);
                    return Err(e);
                }
            };
        }
        2 => {
            // Modo 2: Remover duplicados dentro de um único arquivo
            let file_path = &args[1];
            let temp_output_path = format!("{}.temp", file_path);

            println!("Modo: Remover duplicados em '{}'", file_path);

            match remove_duplicates_in_file(file_path, &temp_output_path) {
                 Ok(count) => {
                    println!("{} linhas únicas mantidas em {}", count, file_path);
                    // Substitui o arquivo original pelo temporário
                    std::fs::rename(&temp_output_path, file_path)?;
                    println!("Duplicados removidos com sucesso em {}.", file_path);
                 }
                 Err(e) => {
                     eprintln!("Erro ao remover duplicados em {}: {}", file_path, e);
                     // Tenta remover o arquivo temporário em caso de erro
                     let _ = std::fs::remove_file(&temp_output_path);
                     return Err(e);
                 }
            }

        }
        _ => {
            // Uso incorreto
            eprintln!("Uso:");
            eprintln!("  {} <arquivo_sent.csv> <arquivo_final.csv> - Remove linhas de final.csv com emails presentes em sent.csv", args[0]);
            eprintln!("  {} <arquivo.csv>                        - Remove linhas duplicadas (baseado na coluna 'email') do arquivo", args[0]);
            return Err("Número incorreto de argumentos".into());
        }
    }

    Ok(())
}

/// Lê um arquivo CSV e retorna um HashSet de emails em minúsculas.
/// Usado no Modo 1 para ler sent.csv.
fn read_emails_to_set(file_path: &str) -> Result<HashSet<String>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut csv_reader = csv::Reader::from_reader(reader);

    let headers = csv_reader.headers()?.clone();
    let email_col_index = headers.iter().position(|h| h == "email");

    let email_col_index = match email_col_index {
        Some(index) => index,
        None => return Err(format!("Coluna 'email' não encontrada em {}", file_path).into()),
    };

    let mut emails = HashSet::new();
    for result in csv_reader.records() {
        let record = result?;
        if let Some(email) = record.get(email_col_index) {
            if !email.trim().is_empty() { // Ignora emails vazios
                 emails.insert(email.trim().to_lowercase()); // Trim para remover espaços em branco
            }
        }
    }

    Ok(emails)
}

/// Processa um arquivo CSV, remove linhas com emails presentes em `emails_to_remove`
/// e escreve o resultado em `temp_output_path`. Usado no Modo 1.
fn clean_file_by_emails(
    emails_to_remove: &HashSet<String>,
    input_file_path: &str,
    temp_output_path: &str,
) -> Result<usize, Box<dyn Error>> {
    let input_file = File::open(input_file_path)?;
    let input_reader = BufReader::new(input_file);
    let mut csv_reader = csv::Reader::from_reader(input_reader);

    let temp_file = File::create(temp_output_path)?;
    let temp_writer = BufWriter::new(temp_file);
    let mut csv_writer = csv::Writer::from_writer(temp_writer);

    // Lê o cabeçalho do arquivo original e escreve no temporário
    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;

    let email_col_index = headers.iter().position(|h| h == "email");

    let email_col_index = match email_col_index {
        Some(index) => index,
        None => {
            return Err(format!("Coluna 'email' não encontrada em {}", input_file_path).into());
        }
    };

    let mut kept_count = 0;
    for result in csv_reader.records() {
        let record = result?;
        let email_cell = record.get(email_col_index);

        let keep_row = match email_cell {
            Some(email) => {
                // Compara em minúsculas e ignora emails vazios
                if email.trim().is_empty() {
                    true // Mantém linhas com email vazio
                } else {
                    !emails_to_remove.contains(&email.trim().to_lowercase())
                }
            }
            None => {
                // Se a célula de email está vazia, decide manter
                true
            }
        };

        if keep_row {
            csv_writer.write_record(&record)?;
            kept_count += 1;
        }
    }

    csv_writer.flush()?;

    Ok(kept_count)
}

/// Lê um arquivo CSV, remove linhas com emails duplicados (mantendo a primeira ocorrência)
/// e escreve o resultado em `temp_output_path`. Usado no Modo 2.
fn remove_duplicates_in_file(
    input_file_path: &str,
    temp_output_path: &str,
) -> Result<usize, Box<dyn Error>> {
    let input_file = File::open(input_file_path)?;
    let input_reader = BufReader::new(input_file);
    let mut csv_reader = csv::Reader::from_reader(input_reader);

    let temp_file = File::create(temp_output_path)?;
    let temp_writer = BufWriter::new(temp_file);
    let mut csv_writer = csv::Writer::from_writer(temp_writer);

    // Lê o cabeçalho do arquivo original e escreve no temporário
    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;

    let email_col_index = headers.iter().position(|h| h == "email");

    let email_col_index = match email_col_index {
        Some(index) => index,
        None => {
            return Err(format!("Coluna 'email' não encontrada em {}", input_file_path).into());
        }
    };

    let mut seen_emails = HashSet::new();
    let mut kept_count = 0;

    for result in csv_reader.records() {
        let record = result?;
        let email_cell = record.get(email_col_index);

        let is_duplicate = match email_cell {
            Some(email) => {
                 if email.trim().is_empty() {
                    // Emails vazios não contam como duplicados para fins de remoção aqui
                    false
                 } else {
                    // Verifica se já vimos este email (em minúsculas)
                    !seen_emails.insert(email.trim().to_lowercase())
                 }
            }
            None => {
                // Linhas sem email não são tratadas como duplicadas neste contexto
                false
            }
        };

        if !is_duplicate {
            // Se não é duplicado (ou é a primeira vez que o vemos), mantém a linha
            csv_writer.write_record(&record)?;
            kept_count += 1;
        }
    }

    csv_writer.flush()?;

    Ok(kept_count)
}
