using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000185 RID: 389
	[HandlerCategory("vvAverages"), HandlerName("LinearRegSlopeMS")]
	public class LinearRegSlopeMS : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000C4F RID: 3151 RVA: 0x000354DC File Offset: 0x000336DC
		public IList<double> Execute(IList<double> src)
		{
			double[] array = new double[src.Count];
			double[] array2 = new double[src.Count];
			double[] array3 = new double[src.Count];
			double[] array4 = new double[src.Count];
			double[] array5 = new double[src.Count];
			double[] array6 = new double[src.Count];
			double[] array7 = new double[src.Count];
			double num = (double)this.Nio + 0.0;
			double num2 = num * (num - 1.0) * 0.5;
			double num3 = (num - 1.0) * num * (2.0 * num - 1.0) / 6.0;
			for (int i = 0; i < src.Count; i++)
			{
				if (i < this.Nio - 1)
				{
					array3[i] = 0.0;
				}
				else
				{
					array3[i] = 100.0 * (src[i] - src[i - (this.Nio - 1)]) / src[i - (this.Nio - 1)] / ((double)this.Nio - 1.0);
				}
			}
			for (int j = 0; j < src.Count; j++)
			{
				if (j < this.Nio)
				{
					array4[j] = 0.0;
				}
				else
				{
					array4[j] = 100.0 * (src[j] - src[j - this.Nio]) / src[j - this.Nio] / ((double)this.Nio - 0.0);
				}
			}
			for (int k = 0; k < src.Count; k++)
			{
				if (k < this.Nio + 1)
				{
					array5[k] = 0.0;
				}
				else
				{
					array5[k] = 100.0 * (src[k] - src[k - (this.Nio + 1)]) / src[k - (this.Nio + 1)] / ((double)this.Nio + 1.0);
				}
			}
			for (int l = 0; l < src.Count; l++)
			{
				if (l < this.Nio + 2)
				{
					array6[l] = 0.0;
				}
				else
				{
					array6[l] = 100.0 * (src[l] - src[l - (this.Nio + 2)]) / src[l - (this.Nio + 2)] / ((double)this.Nio + 2.0);
				}
			}
			for (int m = 0; m < src.Count; m++)
			{
				if (m < this.Nio + 2)
				{
					array7[m] = 0.0;
				}
				else
				{
					array7[m] = (array3[m] / (1.0 + this.Xio) + array4[m] / (this.Xio + 0.0001) + array5[m] / (1.0 - this.Xio) + array6[m] / (2.0 - this.Xio)) / (1.0 / (1.0 + this.Xio) + 1.0 / (this.Xio + 0.0001) + 1.0 / (1.0 - this.Xio) + 1.0 / (2.0 - this.Xio));
				}
			}
			IList<double> list = Series.EMA(array7, 1);
			for (int n = 0; n < src.Count; n++)
			{
				if ((double)n < num)
				{
					array[n] = 0.0;
				}
				else
				{
					double num4 = 0.0;
					double num5 = 0.0;
					int num6 = 0;
					while ((double)num6 <= num - 1.0)
					{
						num4 += (double)num6 * list[n - num6];
						num5 += list[n - num6];
						num6++;
					}
					double num7 = num2 * num5;
					double num8 = num * num4 - num7;
					double num9 = num2 * num2 - num * num3;
					if (num9 != 0.0)
					{
						array[n] = num8 / num9;
					}
					else
					{
						array[n] = 0.0;
					}
				}
			}
			IList<double> list2 = Series.EMA(array, 1);
			for (int num10 = 0; num10 < src.Count; num10++)
			{
				if ((double)num10 < num)
				{
					array2[num10] = 0.0;
				}
				else
				{
					double num11 = 0.0;
					double num12 = 0.0;
					int num13 = 0;
					while ((double)num13 <= num - 1.0)
					{
						num11 += (double)num13 * list2[num10 - num13];
						num12 += list2[num10 - num13];
						num13++;
					}
					double num14 = num2 * num12;
					double num15 = num * num11 - num14;
					double num16 = num2 * num2 - num * num3;
					if (num16 != 0.0)
					{
						array2[num10] = num15 / num16;
					}
					else
					{
						array2[num10] = 0.0;
					}
				}
			}
			return array2;
		}

		// Token: 0x17000408 RID: 1032
		public IContext Context
		{
			// Token: 0x06000C50 RID: 3152 RVA: 0x00035A4A File Offset: 0x00033C4A
			get;
			// Token: 0x06000C51 RID: 3153 RVA: 0x00035A52 File Offset: 0x00033C52
			set;
		}

		// Token: 0x17000406 RID: 1030
		[HandlerParameter(true, "14", Min = "3", Max = "1000", Step = "1")]
		public int Nio
		{
			// Token: 0x06000C4B RID: 3147 RVA: 0x000354BA File Offset: 0x000336BA
			get;
			// Token: 0x06000C4C RID: 3148 RVA: 0x000354C2 File Offset: 0x000336C2
			set;
		}

		// Token: 0x17000407 RID: 1031
		[HandlerParameter(true, "0.5", Min = "0.01", Max = "1", Step = "0.01")]
		public double Xio
		{
			// Token: 0x06000C4D RID: 3149 RVA: 0x000354CB File Offset: 0x000336CB
			get;
			// Token: 0x06000C4E RID: 3150 RVA: 0x000354D3 File Offset: 0x000336D3
			set;
		}
	}
}
