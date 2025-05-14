using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000190 RID: 400
	[HandlerCategory("vvAverages"), HandlerName("NonLagMA")]
	public class NonLagMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000CA4 RID: 3236 RVA: 0x00036D28 File Offset: 0x00034F28
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("NonLagMA", new string[]
			{
				this.Length.ToString(),
				this.PctFilter.ToString(),
				this.Deviation.ToString(),
				src.GetHashCode().ToString()
			}, () => NonLagMA.GenNonLagMA(src, this.Length, this.PctFilter, this.Deviation));
		}

		// Token: 0x06000CA3 RID: 3235 RVA: 0x000369C8 File Offset: 0x00034BC8
		public static IList<double> GenNonLagMA(IList<double> src, int length, double pctfilter, double deviation)
		{
			int count = src.Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double[] array5 = new double[count];
			int num = 4;
			double num2 = 0.0;
			double num3 = 3.1415926535;
			double num4 = 3.0 * num3;
			double num5 = (double)(length - 1);
			double num6 = (double)(length * num) + num5;
			double num7 = 0.0;
			int num8 = 0;
			while ((double)num8 < num6 - 1.0)
			{
				double num9;
				if ((double)num8 <= num5 - 1.0)
				{
					num9 = 1.0 * (double)num8 / (num5 - 1.0);
				}
				else
				{
					num9 = 1.0 + ((double)num8 - num5 + 1.0) * (2.0 * (double)num - 1.0) / ((double)(num * length) - 1.0);
				}
				double num10 = Math.Cos(num3 * num9);
				double num11 = 1.0 / (num4 * num9 + 1.0);
				if (num9 <= 0.5)
				{
					num11 = 1.0;
				}
				array[num8] = num11 * num10;
				num2 += array[num8];
				num8++;
			}
			IList<double> list = LWMA.GenWMA(src, 1);
			int num12 = Convert.ToInt32(num6);
			for (int i = 0; i < num12; i++)
			{
				array4[i] = list[i];
			}
			for (int j = num12; j < count; j++)
			{
				double num13 = 0.0;
				int num14 = 0;
				while ((double)num14 <= num6 - 1.0)
				{
					num13 += array[num14] * list[j - num14];
					num14++;
				}
				if (num2 > 0.0)
				{
					array4[j] = (1.0 + deviation / 100.0) * num13 / num2;
				}
				if (pctfilter > 0.0)
				{
					array2[j] = Math.Abs(array4[j] - array4[j - 1]);
					double num15 = 0.0;
					for (int k = 0; k <= length - 1; k++)
					{
						num15 += array2[j - k];
					}
					array3[j] = num15 / (double)length;
					double num16 = 0.0;
					for (int l = 0; l <= length - 1; l++)
					{
						num16 += Math.Pow(array2[j - l] - array3[j - l], 2.0);
					}
					double num17 = Math.Sqrt(num16 / (double)length);
					num7 = pctfilter * num17;
					if (Math.Abs(array4[j] - array4[j - 1]) < num7)
					{
						array4[j] = array4[j - 1];
					}
				}
				array5[j] = array5[j - 1];
				if (array4[j] - array4[j - 1] > num7)
				{
					array5[j] = 1.0;
				}
				if (array4[j - 1] - array4[j] > num7)
				{
					array5[j] = -1.0;
				}
			}
			return array4;
		}

		// Token: 0x17000421 RID: 1057
		public IContext Context
		{
			// Token: 0x06000CA5 RID: 3237 RVA: 0x00036DB8 File Offset: 0x00034FB8
			get;
			// Token: 0x06000CA6 RID: 3238 RVA: 0x00036DC0 File Offset: 0x00034FC0
			set;
		}

		// Token: 0x17000420 RID: 1056
		[HandlerParameter(true, "0", Min = "0.1", Max = "5", Step = "0.1")]
		public double Deviation
		{
			// Token: 0x06000CA1 RID: 3233 RVA: 0x000369B5 File Offset: 0x00034BB5
			get;
			// Token: 0x06000CA2 RID: 3234 RVA: 0x000369BD File Offset: 0x00034BBD
			set;
		}

		// Token: 0x1700041E RID: 1054
		[HandlerParameter(true, "14", Min = "1", Max = "100", Step = "1")]
		public int Length
		{
			// Token: 0x06000C9D RID: 3229 RVA: 0x00036993 File Offset: 0x00034B93
			get;
			// Token: 0x06000C9E RID: 3230 RVA: 0x0003699B File Offset: 0x00034B9B
			set;
		}

		// Token: 0x1700041F RID: 1055
		[HandlerParameter(true, "0", Min = "0.1", Max = "10", Step = "0.1")]
		public double PctFilter
		{
			// Token: 0x06000C9F RID: 3231 RVA: 0x000369A4 File Offset: 0x00034BA4
			get;
			// Token: 0x06000CA0 RID: 3232 RVA: 0x000369AC File Offset: 0x00034BAC
			set;
		}
	}
}
