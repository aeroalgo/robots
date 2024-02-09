using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000057 RID: 87
	[HandlerCategory("vvIndicators"), HandlerName("Sinewave")]
	public class Sinewave : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600031A RID: 794 RVA: 0x000120EB File Offset: 0x000102EB
		public IList<double> Execute(ISecurity sec)
		{
			return this.GenSinewave(sec, this.Alpha, this.LeadSine, this.Context);
		}

		// Token: 0x06000319 RID: 793 RVA: 0x00011D2C File Offset: 0x0000FF2C
		public IList<double> GenCyclePeriod(ISecurity sec, double alpha)
		{
			int count = sec.get_Bars().Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double[] array5 = new double[count];
			double[] array6 = new double[count];
			double[] array7 = new double[count];
			int num = 5;
			double[] array8 = new double[num];
			for (int i = 6; i < count; i++)
			{
				array3[i] = (this.P(sec, i) + 2.0 * this.P(sec, i - 1) + 2.0 * this.P(sec, i - 2) + this.P(sec, i - 3)) / 6.0;
				array4[i] = (1.0 - 0.5 * this.Alpha) * (1.0 - 0.5 * this.Alpha) * (array3[i] - 2.0 * array3[i - 1] + array3[i - 2]) + 2.0 * (1.0 - this.Alpha) * array4[i - 1] - (1.0 - this.Alpha) * (1.0 - this.Alpha) * array4[i - 2];
				if (i < 8)
				{
					array4[i] = (this.P(sec, i) - 2.0 * this.P(sec, i - 1) + this.P(sec, i - 2)) / 4.0;
				}
				array6[i] = (0.0962 * array4[i] + 0.5769 * array4[i - 2] - 0.5769 * array4[i - 4] - 0.0962 * array4[i - 6]) * (0.5 + 0.08 * array2[i - 1]);
				array7[i] = array4[i - 3];
				if (array6[i] != 0.0 && array6[i - 1] != 0.0)
				{
					array5[i] = (array7[i] / array6[i] - array7[i - 1] / array6[i - 1]) / (1.0 + array7[i] * array7[i - 1] / (array6[i] * array6[i - 1]));
				}
				array5[i] = Math.Max(0.1, array5[i]);
				array5[i] = Math.Min(1.1, array5[i]);
				for (int j = 0; j < num; j++)
				{
					array8[j] = array5[i - j];
				}
				bool flag = false;
				while (!flag)
				{
					for (int k = 1; k < num; k++)
					{
						if (array8[k - 1] > array8[k])
						{
							double num2 = array8[k - 1];
							array8[k - 1] = array8[k];
							array8[k] = num2;
							break;
						}
						if (k == num - 1)
						{
							flag = true;
						}
					}
				}
				double num3;
				if (num % 2 == 0)
				{
					num3 = (array8[num / 2] + array8[num / 2 + 1]) / 2.0;
				}
				else
				{
					num3 = array8[num / 2];
				}
				double num4;
				if (num3 == 0.0)
				{
					num4 = 15.0;
				}
				else
				{
					num4 = 6.28318 / num3 + 0.5;
				}
				array2[i] = 0.33 * num4 + 0.67 * array2[i - 1];
				array[i] = 0.15 * array2[i] + 0.85 * array[i - 1];
			}
			return array;
		}

		// Token: 0x06000317 RID: 791 RVA: 0x000119F0 File Offset: 0x0000FBF0
		public IList<double> GenSinewave(ISecurity sec, double alpha, bool leadsine, IContext context)
		{
			int count = sec.get_Bars().Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double num = Math.Atan(1.0);
			double num2 = 45.0 / num;
			double num3 = 1.0 / num2;
			IList<double> list = this.GenCyclePeriod(sec, alpha);
			double num4 = 0.0;
			for (int i = 0; i < list.Count; i++)
			{
				if (num4 < list[i])
				{
					num4 = list[i];
				}
			}
			int num5 = Convert.ToInt32(num4);
			for (int j = num5; j < count; j++)
			{
				array4[j] = (this.P(sec, j) + 2.0 * this.P(sec, j - 1) + 2.0 * this.P(sec, j - 2) + this.P(sec, j - 3)) / 6.0;
				array3[j] = (1.0 - 0.5 * this.Alpha) * (1.0 - 0.5 * this.Alpha) * (array4[j] - 2.0 * array4[j - 1] + array4[j - 2]) + 2.0 * (1.0 - this.Alpha) * array3[j - 1] - (1.0 - this.Alpha) * (1.0 - this.Alpha) * array3[j - 2];
				int num6 = Convert.ToInt32(Math.Floor(list[j]));
				double num7 = 0.0;
				double num8 = 0.0;
				for (int k = 0; k < num6; k++)
				{
					num7 += Math.Sin(num3 * 360.0 * (double)k / (double)num6) * array3[j - k];
					num8 += Math.Cos(num3 * 360.0 * (double)k / (double)num6) * array3[j - k];
				}
				double num9 = 0.0;
				if (Math.Abs(num8) > 0.001)
				{
					num9 = num2 * Math.Atan(num7 / num8);
				}
				if (Math.Abs(num8) <= 0.001)
				{
					if (num7 >= 0.0)
					{
						num9 = 90.0;
					}
					else
					{
						num9 = -90.0;
					}
				}
				num9 += 90.0;
				if (num8 < 0.0)
				{
					num9 += 180.0;
				}
				if (num9 > 315.0)
				{
					num9 -= 360.0;
				}
				array[j] = Math.Sin(num9 * num3);
				array2[j] = Math.Sin((num9 + 45.0) * num3);
			}
			if (!leadsine)
			{
				return array;
			}
			return array2;
		}

		// Token: 0x06000318 RID: 792 RVA: 0x00011D04 File Offset: 0x0000FF04
		private double P(ISecurity sec, int index)
		{
			return (sec.get_HighPrices()[index] + sec.get_LowPrices()[index]) / 2.0;
		}

		// Token: 0x1700010A RID: 266
		[HandlerParameter(true, "0.07", Min = "0.07", Max = "0.07", Step = "0.01")]
		public double Alpha
		{
			// Token: 0x06000313 RID: 787 RVA: 0x000119CD File Offset: 0x0000FBCD
			get;
			// Token: 0x06000314 RID: 788 RVA: 0x000119D5 File Offset: 0x0000FBD5
			set;
		}

		// Token: 0x1700010C RID: 268
		public IContext Context
		{
			// Token: 0x0600031B RID: 795 RVA: 0x00012106 File Offset: 0x00010306
			get;
			// Token: 0x0600031C RID: 796 RVA: 0x0001210E File Offset: 0x0001030E
			set;
		}

		// Token: 0x1700010B RID: 267
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool LeadSine
		{
			// Token: 0x06000315 RID: 789 RVA: 0x000119DE File Offset: 0x0000FBDE
			get;
			// Token: 0x06000316 RID: 790 RVA: 0x000119E6 File Offset: 0x0000FBE6
			set;
		}
	}
}
