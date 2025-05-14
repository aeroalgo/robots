using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020001A3 RID: 419
	[HandlerCategory("vvAverages"), HandlerName("UllMA")]
	public class UllMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000D46 RID: 3398 RVA: 0x0003A7F8 File Offset: 0x000389F8
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("ullma", new string[]
			{
				this.Type.ToString(),
				src.GetHashCode().ToString()
			}, () => UllMA.GenUllMA(src, this.Type));
		}

		// Token: 0x06000D44 RID: 3396 RVA: 0x0003A63C File Offset: 0x0003883C
		public static IList<double> GenUllMA(IList<double> src, int type)
		{
			int count = src.Count;
			double[] array = new double[count];
			double[] array2 = new double[39];
			double num;
			switch (Math.Abs(type))
			{
			case 0:
				num = 1.25;
				break;
			case 1:
				num = 2.5;
				break;
			case 2:
				num = 3.75;
				break;
			case 3:
				num = 6.25;
				break;
			default:
				num = 6.25;
				break;
			}
			double num2 = 0.0;
			double num3 = 39.0;
			for (int i = 0; i < 39; i++)
			{
				double num4 = UllMA.Sinc((double)i / num3 * (num * 3.1415926535897931));
				array2[i] = num4;
				num2 += num4;
			}
			num2 = 1.0 / num2;
			for (int j = 0; j < 39; j++)
			{
				array2[j] *= num2;
			}
			for (int k = 0; k < count; k++)
			{
				if (k < 39)
				{
					array[k] = src[k];
				}
				else
				{
					double num5 = 0.0;
					for (int l = 0; l < 39; l++)
					{
						num5 += src[k - l] * array2[l];
					}
					array[k] = num5;
				}
			}
			return array;
		}

		// Token: 0x06000D45 RID: 3397 RVA: 0x0003A7A2 File Offset: 0x000389A2
		private static double Sinc(double x)
		{
			if (x == 0.0)
			{
				return 1.0;
			}
			return Math.Sin(3.1415926535897931 * x) / (3.1415926535897931 * x);
		}

		// Token: 0x17000450 RID: 1104
		public IContext Context
		{
			// Token: 0x06000D47 RID: 3399 RVA: 0x0003A864 File Offset: 0x00038A64
			get;
			// Token: 0x06000D48 RID: 3400 RVA: 0x0003A86C File Offset: 0x00038A6C
			set;
		}

		// Token: 0x1700044F RID: 1103
		[HandlerParameter(true, "1", Min = "0", Max = "3", Step = "1")]
		public int Type
		{
			// Token: 0x06000D42 RID: 3394 RVA: 0x0003A629 File Offset: 0x00038829
			get;
			// Token: 0x06000D43 RID: 3395 RVA: 0x0003A631 File Offset: 0x00038831
			set;
		}
	}
}
