#!/bin/bash
set -e

echo "============================================"
echo "  PezMax Backend Build (Java + Maven)"
echo "  Build backend JAR for local deployment"
echo "============================================"
echo ""

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
JAVA_DIR="$ROOT_DIR/PezMax-Java"
DIST_DIR="$ROOT_DIR/build/dist"

# Check Java
if ! command -v java &> /dev/null; then
    if [ -d "/usr/lib/jvm/java-17-openjdk" ]; then
        export JAVA_HOME="/usr/lib/jvm/java-17-openjdk"
        export PATH="$JAVA_HOME/bin:$PATH"
    elif [ -d "/c/Program Files/Java/jdk-17" ]; then
        export JAVA_HOME="/c/Program Files/Java/jdk-17"
        export PATH="$JAVA_HOME/bin:$PATH"
    else
        echo "[ERROR] Java 17+ not found. Install JDK 17+ and set JAVA_HOME."
        exit 1
    fi
fi

mkdir -p "$DIST_DIR"

cd "$JAVA_DIR"

chmod +x mvnw
./mvnw clean package -DskipTests
echo "[OK] Java backend built successfully"

# Copy JAR to dist
JAR_FILE=$(find ruoyi-admin/target -name "*.jar" ! -name "*sources*" ! -name "*javadoc*" ! -name "*.original" 2>/dev/null | head -1)
if [ -n "$JAR_FILE" ]; then
    cp "$JAR_FILE" "$DIST_DIR/ruoyi-admin.jar"
    echo "[OK] Backend JAR copied to $DIST_DIR/ruoyi-admin.jar"
else
    echo "[WARN] JAR not found"
fi

echo "============================================"
echo "  Backend build complete!"
echo "  Output: $DIST_DIR/ruoyi-admin.jar"
echo "  Run: java -jar $DIST_DIR/ruoyi-admin.jar"
echo "============================================"