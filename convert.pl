#!/usr/bin/perl
#
# adh-partnership/chart-parser -- Parse FAA d-TPP chart data for consumption by the ADH
# partnership backend.
#
# (c) 2023 Daniel A. Hawton
#
# This project is licensed by the Apache 2.0 license. See LICENSE for details.
#

use strict;
use warnings;
use diagnostics;

use DateTime;
use DBI;
use Dotenv;
use LWP::UserAgent;
use Time::Moment;
use Time::Piece ();
use Time::Seconds;
use XML::LibXML;

# Check if .env file exists, if so load it!
if (-e ".env") {
  Dotenv->load(".env");
}

my $dbh;
my $today = DateTime->now()->strftime("%Y-%m-%d");

sub plog {
  my $msg = shift;
  my $dt = localtime;
  print '[' . Time::Moment->now->to_string . "] " . $msg . "\n";
}

sub isCycleDate {
  my $dt = shift;

  my $t = Time::Piece->strptime($dt, "%Y-%m-%d");
  my $t2 = Time::Piece->strptime("2023-01-26", "%Y-%m-%d");

  my $diff = $t - $t2;
  my $days = $diff->days;
  return ($days % 28 == 0);
}

sub getFirstCycleDate {
  my $dt = shift;

  my $newdate = "20" . substr($dt, 2, 2) . "-01-01";
  while (!isCycleDate($newdate)) {
    my $t = Time::Piece->strptime($newdate, "%Y-%m-%d");
    $t += ONE_DAY;
    $newdate = $t->strftime("%Y-%m-%d");
  }

  return $newdate;
}

sub numberCyclesInYear {
  my $year = shift;
  my $year_first_cycle = getFirstCycleDate($year . "-01-01");
  my $next_year_first_cycle = getFirstCycleDate(($year + 1) . "-01-01");

  return cyclesBetweenDates($year_first_cycle, $next_year_first_cycle)-1;
}

sub cyclesBetweenDates {
  my $dt1 = shift;
  my $dt2 = shift;

  my $t1 = Time::Piece->strptime($dt1, "%Y-%m-%d");
  my $t2 = Time::Piece->strptime($dt2, "%Y-%m-%d");
  my $diff = $t2 - $t1;
  my $days = $diff->days;
  return int($days / 28) + 1;
}

sub calcCycle {
  my $dt = shift;

  # If $dt is before the first cycle of the year, use the last cycle of last year
  if ($dt lt getFirstCycleDate($dt)) {
    my $year = substr($dt, 2, 2) - 1;
    return sprintf("%02d%02d", $year, numberCyclesInYear(substr($dt, 0, 4)));
  }

  my $year = substr($dt, 2, 2);
  # cycle iteration is defined as the number of 28 day cycles since the first cycle of the year... so let us find that first
  return sprintf("%02d%02d", $year, cyclesBetweenDates(getFirstCycleDate($dt), $dt));
}

sub findDateFromCycle {
  my $cycle = shift;
  my $year = "20" . substr($cycle, 0, 2);
  my $cycleNum = substr($cycle, 2, 2);
  plog "year=$year, cycleNum=$cycleNum";
  my $newdate = $year . "-01-01";
  while (!isCycleDate($newdate)) {
    my $t = Time::Piece->strptime($newdate, "%Y-%m-%d");
    $t += ONE_DAY;
    $newdate = $t->strftime("%Y-%m-%d");
  }
  my $t = Time::Piece->strptime($newdate, "%Y-%m-%d");
  $t += ($cycleNum - 1) * 28 * ONE_DAY;
  my $end = $t + 27 * ONE_DAY;
  return ($t->strftime("%Y-%m-%d"), $end->strftime("%Y-%m-%d"));
}

sub connectDB {
  $dbh = DBI->connect(
    "DBI:mysql:database=" . $ENV{DB_DATABASE} . ";host=" . $ENV{DB_HOST} . ";port=" . $ENV{DB_PORT},
    $ENV{DB_USERNAME},
    $ENV{DB_PASSWORD},
    { RaiseError => 1, AutoCommit => 0 },
  ) or die $DBI::errstr;
}

sub parseData {
  my $cycle = shift;
  my $file = shift;
  my $states = shift(); my @states = @$states;
  my $startdate = shift;
  my $enddate = shift;

  my $query = "INSERT INTO airport_charts (id, airport_id, cycle, from_date, to_date, chart_code, "
    . "chart_name, chart_url, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, NOW(), NOW()) "
    . "ON DUPLICATE KEY UPDATE cycle = ?, from_date = ?, to_date = ?, chart_code = ?, chart_name = ?, "
    . "chart_url = ?, updated_at = NOW()";
  my $sth = $dbh->prepare($query);
  my $dom = XML::LibXML->load_xml(location => $file);
  my $chartCount = 0;
  my $airportCount = 0;
  foreach my $state ($dom->findnodes('/digital_tpp/state_code')) {
    my $stateCode = $state->getAttribute("ID");
    if (grep(/^$stateCode$/, @states) || grep(/^ALL$/i, @states)) {
      foreach my $airport ($state->findnodes("./city_name/airport_name")) {
        $airportCount++;
        foreach my $chart ($airport->findnodes("./record")) {
          my $chartType = $chart->findvalue('./chart_code');
          if (!grep(/^$chartType$/, ("DP", "STAR", "IAP"))) {
            $chartType = "OTHER";
          }
          $sth->execute(
            "FAA-" . $airport->getAttribute("apt_ident") . "-" . $chartType . "-" . $chart->findvalue("./chart_name"),
            $airport->getAttribute("apt_ident"),
            $cycle,
            $startdate,
            $enddate,
            $chartType,
            $chart->findvalue('./chart_name'),
            "https://aeronav.faa.gov/d-tpp/" . $cycle . "/" . $chart->findvalue('./pdf_name'),
            $cycle,
            $startdate,
            $enddate,
            $chartType,
            $chart->findvalue('./chart_name'),
            "https://aeronav.faa.gov/d-tpp/" . $cycle . "/" . $chart->findvalue('./pdf_name'),
          );
          $chartCount++;
        }
      }
    }
  }
  plog "Processed $chartCount charts for $airportCount airports";
}

sub truncateTable {
  my $sth = $dbh->prepare("TRUNCATE TABLE airport_charts");
  $sth->execute();
}

sub deleteWhereCycleNot {
  my $cycle = shift;
  my $sth = $dbh->prepare("DELETE FROM airport_charts WHERE cycle != ?");
  $sth->execute($cycle);
}

plog "ADH Chart Conversion Script";
plog "============================";
plog "Today is " . $today;
plog "Is today a cycle date?";
plog isCycleDate($today) ? "Is Cycle Date" : "Not Cycle Date";
# Check if environment variable FORCE is set
if (!isCycleDate($today)) {
  if ($ENV{FORCE}) {
  plog "FORCE is set, forcing cycle calculation";
  } else {
    plog "FORCE is not set, exiting";
    exit;
  }
}
my @states = $ENV{STATES} ? split(/,/, $ENV{STATES}) : ();
if (scalar(@states) == 0) {
  plog "No states specified, exiting";
  exit;
}
plog "States to process (" . scalar(@states) . "): " . join(", ", @states);
# Call calcCycle with today's date
my $cycle = calcCycle($today);
plog "Cycle is $cycle";
plog "Calculating cycle start date";
my @cycleStartDate = findDateFromCycle($cycle);
plog "Cycle start date is $cycleStartDate[0] - $cycleStartDate[1]";

plog "Connecting to database";
connectDB();

if ($ENV{TRUNCATE}) {
  plog "Truncating database";
  truncateTable;
}

if ($ENV{SKIP_DOWNLOAD}) {
  plog "Skipping download of d-tpp.xml";
} else {
  plog "Downloading d-tpp.xml";
  my $ua = LWP::UserAgent->new;
  my $response = $ua->get('https://aeronav.faa.gov/d-tpp/' . $cycle . '/xml_data/d-tpp_Metafile.xml');
  if ($response->is_success) {
    plog "Saving d-tpp.xml";
    open(my $fh, '>', 'd-tpp.xml');
    print $fh $response->decoded_content;
    close $fh;
  } else {
    plog "Error downloading d-tpp.xml: " . $response->status_line;
    exit;
  }
}

plog "Processing data and inserting to database";
parseData $cycle, 'd-tpp.xml', \@states, @cycleStartDate;

plog "Deleting old data";
deleteWhereCycleNot $cycle;

plog "Committing changes to database";
$dbh->commit();
plog "Done";
