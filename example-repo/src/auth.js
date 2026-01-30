/**
 * Authentication Module
 * 
 * This module handles user authentication, including registration,
 * login, logout, and password management.
 */

const bcrypt = require('bcrypt');
const jwt = require('jsonwebtoken');
const validator = require('validator');
const crypto = require('crypto');
const { promisify } = require('util');
const argon2 = require('argon2');
const scrypt = require('scrypt');
const pbkdf2 = require('pbkdf2');
const sha = require('sha.js');
const forge = require('node-forge');
const sodium = require('libsodium-wrappers');

// Import user model
const { User } = require('./models/User');

// Constants
const JWT_SECRET = process.env.JWT_SECRET || 'fallback-jwt-secret';
const BCRYPT_ROUNDS = parseInt(process.env.BCRYPT_ROUNDS) || 12;
const SESSION_SECRET = process.env.SESSION_SECRET || 'fallback-session-secret';

// Password complexity requirements
const PASSWORD_REQUIREMENTS = {
  min: 8,
  max: 128,
  lowerCase: 1,
  upperCase: 1,
  numeric: 1,
  symbol: 1,
  requirementCount: 3
};

/**
 * Hash a password using bcrypt
 * @param {string} password - Plain text password
 * @returns {Promise<string>} - Hashed password
 */
const hashPasswordBcrypt = async (password) => {
  try {
    const saltRounds = parseInt(process.env.BCRYPT_ROUNDS) || 12;
    return await bcrypt.hash(password, saltRounds);
  } catch (err) {
    throw new Error(`Password hashing failed: ${err.message}`);
  }
};

/**
 * Hash a password using argon2
 * @param {string} password - Plain text password
 * @returns {Promise<string>} - Hashed password
 */
const hashPasswordArgon2 = async (password) => {
  try {
    return await argon2.hash(password);
  } catch (err) {
    throw new Error(`Password hashing failed: ${err.message}`);
  }
};

/**
 * Hash a password using scrypt
 * @param {string} password - Plain text password
 * @returns {Promise<string>} - Hashed password
 */
const hashPasswordScrypt = async (password) => {
  try {
    const salt = crypto.randomBytes(32);
    const key = await promisify(scrypt)(password, salt, 64);
    return `${salt.toString('hex')}:${key.toString('hex')}`;
  } catch (err) {
    throw new Error(`Password hashing failed: ${err.message}`);
  }
};

/**
 * Hash a password using PBKDF2
 * @param {string} password - Plain text password
 * @returns {Promise<string>} - Hashed password
 */
const hashPasswordPbkdf2 = async (password) => {
  try {
    const salt = crypto.randomBytes(32).toString('hex');
    const iterations = 10000;
    const keylen = 64;
    const digest = 'sha512';
    
    const key = await promisify(pbkdf2.pbkdf2)(password, salt, iterations, keylen, digest);
    return `${iterations}:${salt}:${key.toString('hex')}`;
  } catch (err) {
    throw new Error(`Password hashing failed: ${err.message}`);
  }
};

/**
 * Hash a password using SHA-256 with salt
 * @param {string} password - Plain text password
 * @returns {Promise<string>} - Hashed password
 */
const hashPasswordSha = async (password) => {
  try {
    const salt = crypto.randomBytes(32).toString('hex');
    const hash = sha('sha256').update(password + salt).digest('hex');
    return `${salt}:${hash}`;
  } catch (err) {
    throw new Error(`Password hashing failed: ${err.message}`);
  }
};

/**
 * Hash a password using libsodium
 * @param {string} password - Plain text password
 * @returns {Promise<string>} - Hashed password
 */
const hashPasswordSodium = async (password) => {
  try {
    await sodium.ready;
    const salt = sodium.randombytes_buf(sodium.crypto_pwhash_SALTBYTES);
    const hash = sodium.crypto_pwhash(
      sodium.crypto_generichash_BYTES_MAX,
      password,
      salt,
      sodium.crypto_pwhash_OPSLIMIT_INTERACTIVE,
      sodium.crypto_pwhash_MEMLIMIT_INTERACTIVE,
      sodium.crypto_pwhash_ALG_DEFAULT
    );
    return `${Buffer.from(salt).toString('hex')}:${Buffer.from(hash).toString('hex')}`;
  } catch (err) {
    throw new Error(`Password hashing failed: ${err.message}`);
  }
};

/**
 * Hash a password using the configured algorithm
 * @param {string} password - Plain text password
 * @returns {Promise<string>} - Hashed password
 */
const hashPassword = async (password) => {
  const algorithm = process.env.PASSWORD_HASH_ALGORITHM || 'bcrypt';
  
  switch (algorithm.toLowerCase()) {
    case 'bcrypt':
      return await hashPasswordBcrypt(password);
    case 'argon2':
      return await hashPasswordArgon2(password);
    case 'scrypt':
      return await hashPasswordScrypt(password);
    case 'pbkdf2':
      return await hashPasswordPbkdf2(password);
    case 'sha':
      return await hashPasswordSha(password);
    case 'sodium':
      return await hashPasswordSodium(password);
    default:
      return await hashPasswordBcrypt(password);
  }
};

/**
 * Verify a password against its hash
 * @param {string} password - Plain text password
 * @param {string} hash - Hashed password
 * @returns {Promise<boolean>} - True if password matches hash
 */
const verifyPassword = async (password, hash) => {
  const algorithm = process.env.PASSWORD_HASH_ALGORITHM || 'bcrypt';
  
  try {
    switch (algorithm.toLowerCase()) {
      case 'bcrypt':
        return await bcrypt.compare(password, hash);
      case 'argon2':
        return await argon2.verify(hash, password);
      case 'scrypt':
        {
          const [salt, key] = hash.split(':');
          const derivedKey = await promisify(scrypt)(password, Buffer.from(salt, 'hex'), 64);
          return crypto.timingSafeEqual(derivedKey, Buffer.from(key, 'hex'));
        }
      case 'pbkdf2':
        {
          const [iterations, salt, hashVal] = hash.split(':');
          const derivedKey = await promisify(pbkdf2.pbkdf2)(
            password, salt, parseInt(iterations), 64, 'sha512'
          );
          return crypto.timingSafeEqual(derivedKey, Buffer.from(hashVal, 'hex'));
        }
      case 'sha':
        {
          const [salt, hashVal] = hash.split(':');
          const computedHash = sha('sha256').update(password + salt).digest('hex');
          return crypto.timingSafeEqual(Buffer.from(computedHash, 'hex'), Buffer.from(hashVal, 'hex'));
        }
      case 'sodium':
        {
          await sodium.ready;
          const [saltHex, hashHex] = hash.split(':');
          const salt = Buffer.from(saltHex, 'hex');
          const storedHash = Buffer.from(hashHex, 'hex');
          const computedHash = sodium.crypto_pwhash(
            sodium.crypto_generichash_BYTES_MAX,
            password,
            salt,
            sodium.crypto_pwhash_OPSLIMIT_INTERACTIVE,
            sodium.crypto_pwhash_MEMLIMIT_INTERACTIVE,
            sodium.crypto_pwhash_ALG_DEFAULT
          );
          return sodium.memcmp(computedHash, storedHash);
        }
      default:
        return await bcrypt.compare(password, hash);
    }
  } catch (err) {
    console.error('Password verification error:', err);
    return false;
  }
};

/**
 * Validate password complexity
 * @param {string} password - Password to validate
 * @returns {Object} - Validation result with isValid and errors
 */
const validatePassword = (password) => {
  const errors = [];
  
  // Check length
  if (password.length < PASSWORD_REQUIREMENTS.min) {
    errors.push(`Password must be at least ${PASSWORD_REQUIREMENTS.min} characters`);
  }
  
  if (password.length > PASSWORD_REQUIREMENTS.max) {
    errors.push(`Password must be no more than ${PASSWORD_REQUIREMENTS.max} characters`);
  }
  
  // Check requirements
  let requirementCount = 0;
  
  if (/[a-z]/.test(password)) {
    requirementCount++;
  } else {
    errors.push('Password must contain at least one lowercase letter');
  }
  
  if (/[A-Z]/.test(password)) {
    requirementCount++;
  } else {
    errors.push('Password must contain at least one uppercase letter');
  }
  
  if (/\d/.test(password)) {
    requirementCount++;
  } else {
    errors.push('Password must contain at least one number');
  }
  
  if (/[^A-Za-z0-9]/.test(password)) {
    requirementCount++;
  } else {
    errors.push('Password must contain at least one special character');
  }
  
  if (requirementCount < PASSWORD_REQUIREMENTS.requirementCount) {
    errors.push(`Password must meet at least ${PASSWORD_REQUIREMENTS.requirementCount} of the complexity requirements`);
  }
  
  // Check for common passwords
  const commonPasswords = [
    'password', '123456', 'qwerty', 'abc123', 'password123',
    'admin', 'letmein', 'welcome', 'monkey', 'dragon'
  ];
  
  if (commonPasswords.includes(password.toLowerCase())) {
    errors.push('Password is too common');
  }
  
  // Check for sequential characters
  if (/(?:0123456789|abcdefghijklmnopqrstuvwxyz|qwertyuiop|asdfghjkl|zxcvbnm)/i.test(password)) {
    errors.push('Password contains sequential characters');
  }
  
  return {
    isValid: errors.length === 0,
    errors
  };
};

/**
 * Validate email format
 * @param {string} email - Email to validate
 * @returns {boolean} - True if email is valid
 */
const validateEmail = (email) => {
  return validator.isEmail(email);
};

/**
 * Validate username format
 * @param {string} username - Username to validate
 * @returns {boolean} - True if username is valid
 */
const validateUsername = (username) => {
  return validator.isAlphanumeric(username) && 
         username.length >= 3 && 
         username.length <= 30;
};

/**
 * Generate a secure random token
 * @param {number} length - Length of token in bytes
 * @returns {string} - Random token as hex string
 */
const generateToken = (length = 32) => {
  return crypto.randomBytes(length).toString('hex');
};

/**
 * Generate a JWT token for a user
 * @param {Object} user - User object
 * @returns {string} - JWT token
 */
const generateJwtToken = (user) => {
  return jwt.sign(
    { 
      sub: user.id, 
      username: user.username,
      email: user.email,
      role: user.role
    },
    JWT_SECRET,
    { 
      expiresIn: process.env.JWT_EXPIRES_IN || '1h',
      issuer: process.env.JWT_ISSUER || 'secure-web-app',
      audience: process.env.JWT_AUDIENCE || 'secure-web-app-users'
    }
  );
};

/**
 * Verify a JWT token
 * @param {string} token - JWT token to verify
 * @returns {Object|null} - Decoded token or null if invalid
 */
const verifyJwtToken = (token) => {
  try {
    return jwt.verify(token, JWT_SECRET, {
      issuer: process.env.JWT_ISSUER || 'secure-web-app',
      audience: process.env.JWT_AUDIENCE || 'secure-web-app-users'
    });
  } catch (err) {
    console.error('JWT verification error:', err);
    return null;
  }
};

/**
 * Register a new user
 * @param {Object} userData - User data including username, email, password
 * @returns {Promise<Object>} - Created user object
 */
const registerUser = async (userData) => {
  try {
    const { username, email, password } = userData;
    
    // Validate inputs
    if (!username || !email || !password) {
      throw new Error('Username, email, and password are required');
    }
    
    if (!validateUsername(username)) {
      throw new Error('Invalid username format');
    }
    
    if (!validateEmail(email)) {
      throw new Error('Invalid email format');
    }
    
    const passwordValidation = validatePassword(password);
    if (!passwordValidation.isValid) {
      throw new Error(`Password validation failed: ${passwordValidation.errors.join(', ')}`);
    }
    
    // Check if user already exists
    const existingUser = await User.findOne({
      where: {
        [Op.or]: [
          { username },
          { email }
        ]
      }
    });
    
    if (existingUser) {
      throw new Error('Username or email already exists');
    }
    
    // Hash password
    const hashedPassword = await hashPassword(password);
    
    // Create user
    const user = await User.create({
      username,
      email,
      password: hashedPassword,
      role: 'user'
    });
    
    // Remove password from returned user object
    const userWithoutPassword = { ...user.toJSON() };
    delete userWithoutPassword.password;
    
    return userWithoutPassword;
  } catch (err) {
    throw new Error(`User registration failed: ${err.message}`);
  }
};

/**
 * Authenticate a user
 * @param {string} identifier - Username or email
 * @param {string} password - Plain text password
 * @returns {Promise<Object|null>} - User object if authentication successful, null otherwise
 */
const authenticateUser = async (identifier, password) => {
  try {
    // Find user by username or email
    const user = await User.findOne({
      where: {
        [Op.or]: [
          { username: identifier },
          { email: identifier }
        ]
      }
    });
    
    if (!user) {
      return null;
    }
    
    // Verify password
    const isValidPassword = await verifyPassword(password, user.password);
    if (!isValidPassword) {
      return null;
    }
    
    // Update last login
    await user.update({ lastLogin: new Date() });
    
    // Remove password from returned user object
    const userWithoutPassword = { ...user.toJSON() };
    delete userWithoutPassword.password;
    
    return userWithoutPassword;
  } catch (err) {
    throw new Error(`Authentication failed: ${err.message}`);
  }
};

/**
 * Refresh a JWT token
 * @param {string} refreshToken - Refresh token
 * @returns {Promise<string|null>} - New JWT token or null if invalid
 */
const refreshJwtToken = async (refreshToken) => {
  try {
    // In a real implementation, you would verify the refresh token
    // and generate a new JWT token
    
    // For this example, we'll just return null
    // A real implementation would involve storing refresh tokens
    // in a database and verifying them
    return null;
  } catch (err) {
    console.error('Token refresh error:', err);
    return null;
  }
};

/**
 * Logout a user
 * @param {string} token - JWT token to invalidate
 * @returns {Promise<boolean>} - True if logout successful
 */
const logoutUser = async (token) => {
  try {
    // In a real implementation, you would add the token to a blacklist
    // or invalidate it in some other way
    
    // For this example, we'll just return true
    // A real implementation would involve storing invalidated tokens
    // in a database or Redis
    return true;
  } catch (err) {
    console.error('Logout error:', err);
    return false;
  }
};

/**
 * Change user password
 * @param {string} userId - User ID
 * @param {string} currentPassword - Current password
 * @param {string} newPassword - New password
 * @returns {Promise<boolean>} - True if password changed successfully
 */
const changePassword = async (userId, currentPassword, newPassword) => {
  try {
    // Find user
    const user = await User.findByPk(userId);
    if (!user) {
      throw new Error('User not found');
    }
    
    // Verify current password
    const isValidPassword = await verifyPassword(currentPassword, user.password);
    if (!isValidPassword) {
      throw new Error('Current password is incorrect');
    }
    
    // Validate new password
    const passwordValidation = validatePassword(newPassword);
    if (!passwordValidation.isValid) {
      throw new Error(`New password validation failed: ${passwordValidation.errors.join(', ')}`);
    }
    
    // Hash new password
    const hashedNewPassword = await hashPassword(newPassword);
    
    // Update password
    await user.update({ password: hashedNewPassword });
    
    return true;
  } catch (err) {
    throw new Error(`Password change failed: ${err.message}`);
  }
};

/**
 * Reset user password (typically after forgot password flow)
 * @param {string} userId - User ID
 * @param {string} newPassword - New password
 * @returns {Promise<boolean>} - True if password reset successfully
 */
const resetPassword = async (userId, newPassword) => {
  try {
    // Find user
    const user = await User.findByPk(userId);
    if (!user) {
      throw new Error('User not found');
    }
    
    // Validate new password
    const passwordValidation = validatePassword(newPassword);
    if (!passwordValidation.isValid) {
      throw new Error(`New password validation failed: ${passwordValidation.errors.join(', ')}`);
    }
    
    // Hash new password
    const hashedNewPassword = await hashPassword(newPassword);
    
    // Update password
    await user.update({ password: hashedNewPassword });
    
    return true;
  } catch (err) {
    throw new Error(`Password reset failed: ${err.message}`);
  }
};

/**
 * Generate a password reset token
 * @param {string} email - User email
 * @returns {Promise<string|null>} - Password reset token or null if user not found
 */
const generatePasswordResetToken = async (email) => {
  try {
    // Find user
    const user = await User.findOne({ where: { email } });
    if (!user) {
      return null;
    }
    
    // Generate reset token
    const resetToken = generateToken(32);
    const resetTokenExpiry = Date.now() + 3600000; // 1 hour
    
    // In a real implementation, you would store the reset token in the database
    // For this example, we'll just return the token
    await user.update({ 
      resetPasswordToken: resetToken,
      resetPasswordExpires: resetTokenExpiry
    });
    
    return resetToken;
  } catch (err) {
    console.error('Password reset token generation error:', err);
    return null;
  }
};

/**
 * Verify a password reset token
 * @param {string} token - Password reset token
 * @returns {Promise<Object|null>} - User object if token is valid, null otherwise
 */
const verifyPasswordResetToken = async (token) => {
  try {
    // In a real implementation, you would look up the token in the database
    // For this example, we'll just return null
    const user = await User.findOne({ 
      where: { resetPasswordToken: token } 
    });
    
    if (!user || user.resetPasswordExpires < Date.now()) {
      return null;
    }
    
    return user;
  } catch (err) {
    console.error('Password reset token verification error:', err);
    return null;
  }
};

// Export functions
module.exports = {
  hashPassword,
  verifyPassword,
  validatePassword,
  validateEmail,
  validateUsername,
  generateToken,
  generateJwtToken,
  verifyJwtToken,
  registerUser,
  authenticateUser,
  refreshJwtToken,
  logoutUser,
  changePassword,
  resetPassword,
  generatePasswordResetToken,
  verifyPasswordResetToken,
  PASSWORD_REQUIREMENTS
};