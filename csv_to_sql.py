import pandas as pd

# Load the CSV data into a pandas DataFrame
df = pd.read_csv("db/upstream.csv")

# Create a mapping of telegram_id to a sequential user_id and url to a sequential website_id
telegram_id_to_sequential_id = {telegram_id: idx + 1 for idx, telegram_id in enumerate(df['telegram_id'].unique())}
url_to_sequential_id = {url: idx + 1 for idx, url in enumerate(df['url'].unique())}

# Generate SQL statements for users table using sequential ids
users_sql_statements = [f"INSERT INTO users (id, name, user_type, telegram_id) VALUES ({telegram_id_to_sequential_id[telegram_id]}, NULL, NULL, {telegram_id});" 
                        for telegram_id in df['telegram_id'].unique()]

# Generate SQL statements for websites table using sequential ids
websites_sql_statements = [f"INSERT INTO websites (id, last_checked_time, status, url) VALUES ({url_to_sequential_id[row['url']]}, '{row['last_checked_time']}', {row['status']}, '{row['url']}');" 
                           for _, row in df.drop_duplicates(subset='url').iterrows()]

# Generate SQL statements for user_websites table using the sequential ids for relationships
user_websites_sql_statements = [f"INSERT INTO user_websites (user_id, website_id) VALUES ({telegram_id_to_sequential_id[row['telegram_id']]}, {url_to_sequential_id[row['url']]});" 
                                for _, row in df.iterrows()]

# Combine all the regenerated SQL statements into one list
all_sql_statements = users_sql_statements + websites_sql_statements + user_websites_sql_statements

# Write the combined SQL statements to a .sql file
with open("db/migrate.sql", "w") as output_file:
    output_file.write("\n".join(all_sql_statements))
