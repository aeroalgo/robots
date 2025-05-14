using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200016A RID: 362
	[HandlerCategory("vvAverages"), HandlerName("FIR MA")]
	public class FIR_MA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000B82 RID: 2946 RVA: 0x0002F0FC File Offset: 0x0002D2FC
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("FIR_MA", new string[]
			{
				this.Periods.ToString(),
				this.Taps.ToString(),
				this.Window.ToString(),
				src.GetHashCode().ToString()
			}, () => FIR_MA.GenFIR_MA(src, this.Periods, this.Taps, this.Window));
		}

		// Token: 0x06000B81 RID: 2945 RVA: 0x0002EE38 File Offset: 0x0002D038
		public static IList<double> GenFIR_MA(IList<double> src, int periods, int taps, int window)
		{
			int count = src.Count;
			if (taps % 2 == 0)
			{
				taps++;
			}
			if (count < taps)
			{
				return null;
			}
			double[] array = new double[count];
			double[] array2 = new double[taps];
			double num = 3.1415926535897931;
			double num2 = 0.0;
			for (int i = 0; i < taps; i++)
			{
				switch (window)
				{
				case 1:
					array2[i] = 1.0;
					break;
				case 2:
					array2[i] = 0.5 - 0.5 * Math.Cos(2.0 * num * (double)i / (double)taps);
					break;
				case 3:
					array2[i] = 0.54 - 0.46 * Math.Cos(2.0 * num * (double)i / (double)taps);
					break;
				case 4:
					array2[i] = 0.42 - 0.5 * Math.Cos(2.0 * num * (double)i / (double)taps) + 0.08 * Math.Cos(4.0 * num * (double)i / (double)taps);
					break;
				case 5:
					array2[i] = 0.35875 - 0.48829 * Math.Cos(2.0 * num * (double)i / (double)taps) + 0.14128 * Math.Cos(4.0 * num * (double)i / (double)taps) - 0.01168 * Math.Cos(6.0 * num * (double)i / (double)taps);
					break;
				default:
					array2[i] = 1.0;
					break;
				}
				if ((double)i != (double)taps / 2.0)
				{
					array2[i] = array2[i] * Math.Sin(num * ((double)i - (double)taps / 2.0) / (double)periods) / num / ((double)i - (double)taps / 2.0);
				}
				num2 += array2[i];
			}
			for (int j = 0; j < count; j++)
			{
				array[j] = 0.0;
				if (j < taps)
				{
					array[j] = src[j];
				}
				else
				{
					for (int k = 0; k < taps; k++)
					{
						array[j] += src[j - k] * array2[k] / num2;
					}
				}
			}
			return array;
		}

		// Token: 0x170003CC RID: 972
		public IContext Context
		{
			// Token: 0x06000B83 RID: 2947 RVA: 0x0002F18C File Offset: 0x0002D38C
			get;
			// Token: 0x06000B84 RID: 2948 RVA: 0x0002F194 File Offset: 0x0002D394
			set;
		}

		// Token: 0x170003C9 RID: 969
		[HandlerParameter(true, "7", Min = "3", Max = "15", Step = "1")]
		public int Periods
		{
			// Token: 0x06000B7B RID: 2939 RVA: 0x0002EE05 File Offset: 0x0002D005
			get;
			// Token: 0x06000B7C RID: 2940 RVA: 0x0002EE0D File Offset: 0x0002D00D
			set;
		}

		// Token: 0x170003CA RID: 970
		[HandlerParameter(true, "15", Min = "7", Max = "25", Step = "2", Name = "д.б. нечётным")]
		public int Taps
		{
			// Token: 0x06000B7D RID: 2941 RVA: 0x0002EE16 File Offset: 0x0002D016
			get;
			// Token: 0x06000B7E RID: 2942 RVA: 0x0002EE1E File Offset: 0x0002D01E
			set;
		}

		// Token: 0x170003CB RID: 971
		[HandlerParameter(true, "4", Min = "1", Max = "5", Step = "1", Name = "1-прямоугольное окно\n2-Hanning,3-Hamming\n4-Blackman\n5-Blackman-Harris")]
		public int Window
		{
			// Token: 0x06000B7F RID: 2943 RVA: 0x0002EE27 File Offset: 0x0002D027
			get;
			// Token: 0x06000B80 RID: 2944 RVA: 0x0002EE2F File Offset: 0x0002D02F
			set;
		}
	}
}
