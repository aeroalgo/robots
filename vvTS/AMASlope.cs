using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000159 RID: 345
	[HandlerCategory("vvAverages"), HandlerName("AMASlope")]
	public class AMASlope : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000ADB RID: 2779 RVA: 0x0002CCDA File Offset: 0x0002AEDA
		public IList<double> Execute(IList<double> src)
		{
			return AMASlope.GenAMASlope(src, this.AMAPeriod, this.nfast, this.nslow);
		}

		// Token: 0x06000ADA RID: 2778 RVA: 0x0002CBBC File Offset: 0x0002ADBC
		public static IList<double> GenAMASlope(IList<double> src, int amaperiod, int _nfast, int _nslow)
		{
			int num = amaperiod + 2;
			int count = src.Count;
			if (count < num)
			{
				return null;
			}
			double[] array = new double[count];
			double y = 2.0;
			double num2 = 2.0 / (double)(_nslow + 1);
			double num3 = 2.0 / (double)(_nfast + 1);
			double num4 = src[0];
			for (int i = amaperiod + 2; i < count; i++)
			{
				double num5 = Math.Abs(src[i] - src[i - amaperiod]);
				double num6 = 1E-09;
				for (int j = 0; j < amaperiod; j++)
				{
					num6 += Math.Abs(src[i - j] - src[i - j - 1]);
				}
				double num7 = num5 / num6;
				double num8 = num3 - num2;
				double num9 = num7 * num8;
				double x = num9 + num2;
				double num10 = num4 + Math.Pow(x, y) * (src[i] - num4);
				double num11 = num10 - num4;
				array[i] = num11;
				num4 = num10;
			}
			return array;
		}

		// Token: 0x17000396 RID: 918
		[HandlerParameter(true, "20", Min = "10", Max = "100", Step = "5")]
		public int AMAPeriod
		{
			// Token: 0x06000AD4 RID: 2772 RVA: 0x0002CB88 File Offset: 0x0002AD88
			get;
			// Token: 0x06000AD5 RID: 2773 RVA: 0x0002CB90 File Offset: 0x0002AD90
			set;
		}

		// Token: 0x17000397 RID: 919
		[HandlerParameter(true, "2", Min = "1", Max = "10", Step = "1")]
		public int nfast
		{
			// Token: 0x06000AD6 RID: 2774 RVA: 0x0002CB99 File Offset: 0x0002AD99
			get;
			// Token: 0x06000AD7 RID: 2775 RVA: 0x0002CBA1 File Offset: 0x0002ADA1
			set;
		}

		// Token: 0x17000398 RID: 920
		[HandlerParameter(true, "30", Min = "1", Max = "50", Step = "1")]
		public int nslow
		{
			// Token: 0x06000AD8 RID: 2776 RVA: 0x0002CBAA File Offset: 0x0002ADAA
			get;
			// Token: 0x06000AD9 RID: 2777 RVA: 0x0002CBB2 File Offset: 0x0002ADB2
			set;
		}
	}
}
