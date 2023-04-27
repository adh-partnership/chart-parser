FROM perl:5.36-bullseye

# Create app directory
WORKDIR /usr/src/app

# Install dependences
RUN apt-get update && apt-get install -y \
    libdbd-mysql-perl \
    libdbi-perl \
    libmariadb-dev \
    && rm -rf /var/lib/apt/lists/* \
    && cpanm install DateTime DBI Dotenv LWP::UserAgent Time::Moment Time::Piece Time::Seconds XML::LibXML

COPY convert.pl ./

CMD [ "perl", "convert.pl" ]
