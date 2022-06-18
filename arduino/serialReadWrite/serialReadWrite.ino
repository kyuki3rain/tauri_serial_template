void setup() {
  Serial.begin(9600);
}

void loop() {
  String line;
  long line_len, num;
  char input;
  
  if (Serial.available() > 0) {
    line = "";
    while(Serial.available() > 0) {
      input = Serial.read();
      if(!isDigit(input)) { continue; }
      line.concat(input);
    }
    line_len = line.length();

    if (line_len > 0) {
        num = line.toInt();
        num *= 2;
        Serial.println(num);
    }
  }

  delay(100);
}
