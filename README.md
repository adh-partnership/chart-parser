# ADH Chart Parser

Calculates cycle number and grabs the data from the FAA d-TPP Metafile.

## Configuration

The script depends on the following environment variables to function:

```bash
DB_USERNAME=<username>
DB_PASSWORD=<password>
DB_HOST=<hostname>
DB_PORT=<port>
DB_DATABASE=<database>
STATES=<state1>,<state2>,<state3>
```

STATES can be a comma-separated list of states, or a single state. If you wish to
import all of the chart data, you can also specify "ALL".

*NOTE*: The FAA puts Oceana charts for things like Guam, the Marshal Islands, etc.
under state "XX".

Optional environment variables:

- SKIP_DOWNLOAD - If you already have the d-tpp.xml file downloaded, you can set this to 1 to skip the download step.
- TRUNCATE - Truncate the airport_charts table before importing. This is useful if you want to re-import all of the charts.
- FORCE - Force the import, even if today is not the start of a new cycle. It will calculate the cycle number based on the current date.

## Usage

You can specify the environment variables in the environment or by created a .env. Then:

```bash
perl convert.pl
```

This was built with the idea of being run daily, so it will only import charts when a new cycle
begins. If you want to force the import, you can set the FORCE environment variable.

## Requirements

This script requires the following Perl modules:

- DateTime
- DBI
- DBD::MySQL
- Dotenv
- LWP::UserAgent
- Time::Moment
- Time::Piece
- Time::Seconds
- XML::LibXML

## License

This script is licensed under the Apache 2.0 license. See the LICENSE file for more information.
