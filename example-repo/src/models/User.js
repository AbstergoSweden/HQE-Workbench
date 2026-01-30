/**
 * User Model
 * 
 * Defines the User model with validations, hooks, and associations.
 */

const { DataTypes, Model } = require('sequelize');
const { Op } = require('sequelize');
const sequelize = require('../config/database');
const bcrypt = require('bcrypt');
const crypto = require('crypto');

class User extends Model {
  /**
   * Check if email is taken
   * @param {string} email - The user's email
   * @param {ObjectId} [excludeUserId] - The id of the user to be excluded
   * @returns {Promise<boolean>}
   */
  static async isEmailTaken(email, excludeUserId) {
    const user = await this.findOne({
      where: {
        email,
        ...(excludeUserId && { id: { [Op.ne]: excludeUserId } })
      }
    });
    return !!user;
  }

  /**
   * Check if username is taken
   * @param {string} username - The user's username
   * @param {ObjectId} [excludeUserId] - The id of the user to be excluded
   * @returns {Promise<boolean>}
   */
  static async isUsernameTaken(username, excludeUserId) {
    const user = await this.findOne({
      where: {
        username,
        ...(excludeUserId && { id: { [Op.ne]: excludeUserId } })
      }
    });
    return !!user;
  }

  /**
   * Check if password matches the user's password
   * @param {string} password
   * @returns {Promise<boolean>}
   */
  async isPasswordMatch(password) {
    const hashedPassword = this.password;
    return bcrypt.compare(password, hashedPassword);
  }

  /**
   * Generate reset password token
   * @returns {string}
   */
  generateResetPasswordToken() {
    const resetPasswordToken = crypto.randomBytes(32).toString('hex');
    this.resetPasswordToken = resetPasswordToken;
    this.resetPasswordExpires = Date.now() + 3600000; // 1 hour
    return resetPasswordToken;
  }

  /**
   * Generate verify email token
   * @returns {string}
   */
  generateVerifyEmailToken() {
    const verifyEmailToken = crypto.randomBytes(32).toString('hex');
    this.verifyEmailToken = verifyEmailToken;
    this.verifyEmailExpires = Date.now() + 24 * 60 * 60 * 1000; // 24 hours
    return verifyEmailToken;
  }
}

User.init({
  id: {
    type: DataTypes.UUID,
    defaultValue: DataTypes.UUIDV4,
    primaryKey: true
  },
  username: {
    type: DataTypes.STRING,
    allowNull: false,
    unique: true,
    validate: {
      len: [3, 30],
      isAlphanumeric: true,
      notEmpty: true
    }
  },
  email: {
    type: DataTypes.STRING,
    allowNull: false,
    unique: true,
    validate: {
      isEmail: true,
      notEmpty: true
    }
  },
  password: {
    type: DataTypes.STRING,
    allowNull: false,
    validate: {
      len: [8, 128],
      notEmpty: true
    },
    set(value) {
      // Hash password before saving
      const saltRounds = parseInt(process.env.BCRYPT_ROUNDS) || 12;
      const hashedPassword = bcrypt.hashSync(value, saltRounds);
      this.setDataValue('password', hashedPassword);
    }
  },
  role: {
    type: DataTypes.ENUM('user', 'admin', 'moderator'),
    defaultValue: 'user',
    allowNull: false
  },
  isActive: {
    type: DataTypes.BOOLEAN,
    defaultValue: true,
    allowNull: false
  },
  lastLogin: {
    type: DataTypes.DATE
  },
  resetPasswordToken: {
    type: DataTypes.STRING,
    allowNull: true
  },
  resetPasswordExpires: {
    type: DataTypes.DATE,
    allowNull: true
  },
  verifyEmailToken: {
    type: DataTypes.STRING,
    allowNull: true
  },
  verifyEmailExpires: {
    type: DataTypes.DATE,
    allowNull: true
  },
  emailVerified: {
    type: DataTypes.BOOLEAN,
    defaultValue: false,
    allowNull: false
  },
  profilePicture: {
    type: DataTypes.STRING,
    allowNull: true
  },
  firstName: {
    type: DataTypes.STRING,
    allowNull: true,
    validate: {
      len: [1, 50]
    }
  },
  lastName: {
    type: DataTypes.STRING,
    allowNull: true,
    validate: {
      len: [1, 50]
    }
  },
  bio: {
    type: DataTypes.TEXT,
    allowNull: true,
    validate: {
      len: [0, 500]
    }
  },
  phone: {
    type: DataTypes.STRING,
    allowNull: true,
    validate: {
      is: /^[\+]?[1-9][\d]{0,15}$/
    }
  },
  timezone: {
    type: DataTypes.STRING,
    defaultValue: 'UTC',
    allowNull: false
  },
  language: {
    type: DataTypes.STRING,
    defaultValue: 'en',
    allowNull: false
  },
  twoFactorEnabled: {
    type: DataTypes.BOOLEAN,
    defaultValue: false,
    allowNull: false
  },
  twoFactorSecret: {
    type: DataTypes.STRING,
    allowNull: true
  },
  twoFactorBackupCodes: {
    type: DataTypes.JSONB,
    allowNull: true
  },
  loginAttempts: {
    type: DataTypes.INTEGER,
    defaultValue: 0,
    allowNull: false,
    validate: {
      min: 0,
      max: 10
    }
  },
  lockedUntil: {
    type: DataTypes.DATE,
    allowNull: true
  },
  createdAt: {
    type: DataTypes.DATE,
    defaultValue: DataTypes.NOW,
    allowNull: false
  },
  updatedAt: {
    type: DataTypes.DATE,
    defaultValue: DataTypes.NOW,
    allowNull: false
  }
}, {
  sequelize,
  modelName: 'User',
  tableName: 'users',
  timestamps: true,
  paranoid: true, // Enable soft deletes
  indexes: [
    {
      unique: true,
      fields: ['email']
    },
    {
      unique: true,
      fields: ['username']
    },
    {
      fields: ['role']
    },
    {
      fields: ['isActive']
    },
    {
      fields: ['emailVerified']
    },
    {
      fields: ['createdAt']
    }
  ],
  hooks: {
    beforeCreate: async (user) => {
      // Validate email uniqueness
      const existingUser = await User.findOne({
        where: { email: user.email }
      });
      
      if (existingUser) {
        throw new Error('Email already taken');
      }
      
      // Validate username uniqueness
      const existingUsername = await User.findOne({
        where: { username: user.username }
      });
      
      if (existingUsername) {
        throw new Error('Username already taken');
      }
    },
    
    beforeUpdate: async (user) => {
      // If email is being updated, validate uniqueness
      if (user.changed('email')) {
        const existingUser = await User.findOne({
          where: { 
            email: user.email,
            id: { [Op.ne]: user.id }
          }
        });
        
        if (existingUser) {
          throw new Error('Email already taken');
        }
      }
      
      // If username is being updated, validate uniqueness
      if (user.changed('username')) {
        const existingUsername = await User.findOne({
          where: { 
            username: user.username,
            id: { [Op.ne]: user.id }
          }
        });
        
        if (existingUsername) {
          throw new Error('Username already taken');
        }
      }
      
      // If password is being updated, hash it
      if (user.changed('password')) {
        const saltRounds = parseInt(process.env.BCRYPT_ROUNDS) || 12;
        const hashedPassword = bcrypt.hashSync(user.password, saltRounds);
        user.password = hashedPassword;
      }
    },
    
    beforeDestroy: (user, options) => {
      // Prevent deletion of admin users
      if (user.role === 'admin') {
        throw new Error('Cannot delete admin user');
      }
    }
  }
});

// Define associations
User.associate = (models) => {
  // A user can have many posts
  User.hasMany(models.Post, {
    foreignKey: 'authorId',
    as: 'posts'
  });
  
  // A user can have many comments
  User.hasMany(models.Comment, {
    foreignKey: 'userId',
    as: 'comments'
  });
  
  // A user can have many sessions
  User.hasMany(models.Session, {
    foreignKey: 'userId',
    as: 'sessions'
  });
  
  // A user can have many login attempts
  User.hasMany(models.LoginAttempt, {
    foreignKey: 'userId',
    as: 'loginAttemptsHistory'
  });
};

module.exports = User;